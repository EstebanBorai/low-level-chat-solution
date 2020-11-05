use mio::{Handler, Token, EventLoop, EventSet, PollOpt};
use mio::tcp::{TcpListener, TcpStream};
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(0);

type Clients = HashMap<Token, TcpStream>;

pub struct WebSocketServer {
  pub socket: TcpListener,
  clients: Clients,
  token_counter: usize,
}

impl WebSocketServer {
  pub fn new(socket: TcpListener) -> Self {
    Self {
      socket,
      token_counter: 1,
      clients: HashMap::new(),
    }
  }
}

impl Handler for WebSocketServer {
  type Timeout = usize;
  type Message = ();

  fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
      match token {
        SERVER_TOKEN => {
          let client_socket = match self.socket.accept() {
            Err(e) => {
              println!("Accept Error: {}", e);
              return;
            },
            Ok(None) => unreachable!("Accept returned `None`"),
            Ok(Some((socket, _))) => socket
          };

          self.token_counter += 1;
          let new_token = Token(self.token_counter);

          self.clients.insert(new_token, client_socket);

          event_loop.register(&self.clients[&new_token],
            new_token,
            EventSet::readable(),
            PollOpt::edge() | PollOpt::oneshot()).unwrap();
        },
        _ => println!("Received invalid token: {:?}", token)
      }
  }
}
