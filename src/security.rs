use rustc_serialize::base64::{ToBase64, STANDARD};
use sha1::Sha1;

pub fn gen_key(key: &String) -> String {
    let mut m = Sha1::new();
    let mut buff = [0u8; 20];

    m.update(key.as_bytes());
    m.update("258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes());

    m.output(&mut buff);

    buff.to_base64(STANDARD)
}
