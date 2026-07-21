use crate::Client;
use haya_protocol::clientbound::{LoginHandler, cookie, login};
use haya_protocol::serverbound::login::LoginAcknowledged;
use mser::{Read, Reader, V21};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub async fn handle_login(
    client: &mut Client,
    b: &mut Vec<u8>,
    c: &mut usize,
) -> tokio::io::Result<(Uuid, Box<str>)> {
    let mut flag = true;
    let (read, p) = loop {
        if !flag {
            b.reserve(4096);
            client.s.read_buf(b).await?;
        }
        flag = false;
        let buf = &b[*c..];
        let ptr = buf.as_ptr();
        let mut r = Reader::new(buf);
        let len = match V21::read(&mut r) {
            Ok(x) => x.0 as usize,
            Err(_) => continue,
        };
        match r.read_slice(len) {
            Ok(x) => unsafe { break (r.offset_from(ptr), x) },
            Err(_) => continue,
        }
    };
    *c += read;
    let p = Reader::new(p);
    let mut h = Login {
        id: Uuid::nil(),
        name: None,
        c: client,
    };
    h.handle(p).unwrap();
    h.c.flush().await?;
    Ok((
        h.id,
        match h.name {
            Some(x) => x,
            None => return Err(tokio::io::ErrorKind::Other.into()),
        },
    ))
}

pub struct Login<'a> {
    id: Uuid,
    name: Option<Box<str>>,
    c: &'a mut Client,
}

impl<'a> LoginHandler for Login<'a> {
    fn login_disconnect(&mut self, _: login::LoginDisconnect<'_>) {}
    fn hello(&mut self, _: login::Hello<'_>) {}
    fn login_finished(&mut self, packet: login::LoginFinished<'_>) {
        self.name = Some(packet.game_profile.name.0.to_owned().into_boxed_str());
        self.id = packet.game_profile.id;
        self.c.write(&LoginAcknowledged {});
    }
    fn login_compression(&mut self, _: login::LoginCompression) {}
    fn custom_query(&mut self, _: login::CustomQuery<'_>) {}
    fn cookie_request(&mut self, _: cookie::LoginCookieRequest<'_>) {}
}
