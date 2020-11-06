use http_muncher::{Parser, ParserHandler};
use mio::tcp::*;
use mio::EventSet;
use mio::{TryRead, TryWrite};
use rustc_serialize::base64::{ToBase64, STANDARD};
use sha1::Sha1;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::from_utf8;
use std::fmt;

pub struct HttpParser {
    current_key: Option<String>,
    headers: Rc<RefCell<HashMap<String, String>>>
}

impl ParserHandler for HttpParser {
    fn on_header_field(&mut self, bytes: &[u8]) -> bool {
        self.current_key = Some(from_utf8(bytes).unwrap().to_string());

        true
    }

    fn on_header_value(&mut self, bytes: &[u8]) -> bool {
        self.headers.borrow_mut().insert(
            self.current_key.clone().unwrap(),
            from_utf8(bytes).unwrap().to_string(),
        );

        true
    }

    fn on_headers_complete(&mut self) -> bool {
        false
    }
}

#[derive(PartialEq)]
pub enum ClientState {
    AwaitingHandshake,
    HandshakeResponse,
    Connected,
}

pub struct WebSocketClient {
    pub socket: TcpStream,
    pub http_parser: Parser<HttpParser>,
    pub headers: Rc<RefCell<HashMap<String, String>>>,
    pub interest: EventSet,
    pub state: ClientState,
}

impl WebSocketClient {
    pub fn new(socket: TcpStream) -> Self {
        let headers = Rc::new(RefCell::new(HashMap::new()));

        Self {
            socket,
            interest: EventSet::readable(),
            state: ClientState::AwaitingHandshake,
            http_parser: Parser::request(HttpParser {
                current_key: None,
                headers: headers.clone(),
            }),
            headers,
        }
    }

    pub fn read(&mut self) {
        loop {
            let mut buff = [0; 2048];

            match self.socket.try_read(&mut buff) {
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return;
                }
                Ok(None) => break,
                Ok(Some(len)) => {
                    self.http_parser.parse(&buff[0..len]);

                    if self.http_parser.is_upgrade() {
                        self.state = ClientState::HandshakeResponse;
                        self.interest.remove(EventSet::readable());
                        self.interest.insert(EventSet::writable());
                        break;
                    }
                }
            }
        }
    }

    pub fn write(&mut self) {
        let headers = self.headers.borrow();
        let response_key = Self::gen_key(headers.get("Sec-WebSocket-Key").unwrap());
        let response = fmt::format(format_args!("HTTP/1.1 101 Switching Protocols\r\n\
                                              Connection: Upgrade\r\n\
                                              Sec-WebSocket-Accept: {}\r\n\
                                              Upgrade: websocket\r\n\r\n", response_key));

        self.socket.try_write(response.as_bytes()).unwrap();
        self.state = ClientState::Connected;
        self.interest.remove(EventSet::writable());
        self.interest.insert(EventSet::readable());
    }

    fn gen_key(key: &String) -> String {
        let mut m = Sha1::new();
        let mut buff = [0u8; 20];
    
        m.update(key.as_bytes());
        m.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());
    
        m.output(&mut buff);
    
        buff.to_base64(STANDARD)
    }
}
