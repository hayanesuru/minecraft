extern crate alloc;

mod configuration;
mod game;
mod login;

use self::configuration::handle_configuration;
use self::game::handle_game;
use self::login::handle_login;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use core::sync::atomic::AtomicBool;
use core::time::Duration;
use haya_protocol::serverbound::handshake::{ClientIntent, Intention};
use haya_protocol::serverbound::login::Hello;
use haya_protocol::types::{Id, packet_id};
use mser::{Utf8, V21, V32, Write, write_unchecked};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::task::spawn;
use tokio::time::sleep;
use uuid::Uuid;

fn main() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());
}

static STOP: AtomicBool = AtomicBool::new(false);

struct Client {
    s: TcpStream,
    b: Vec<u8>,
}

impl Client {
    fn write(&mut self, p: &(impl Write + ?Sized + Id)) {
        let pl = p.len_s();
        let i = packet_id(p);
        let il = i.len_s();
        let header = V21((pl + il) as u32);
        let hl = header.len_s();
        unsafe {
            self.b.reserve(hl + pl + il);
            write_unchecked(self.b.as_mut_ptr().add(self.b.len()), &header);
            self.b.set_len(self.b.len() + hl);
            write_unchecked(self.b.as_mut_ptr().add(self.b.len()), &i);
            self.b.set_len(self.b.len() + il);
            write_unchecked(self.b.as_mut_ptr().add(self.b.len()), p);
            self.b.set_len(self.b.len() + pl);
        }
    }

    async fn flush(&mut self) -> tokio::io::Result<()> {
        self.s.write_all(&self.b).await?;
        self.b.clear();
        Ok(())
    }
}

async fn run() {
    spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
        STOP.store(true, core::sync::atomic::Ordering::Relaxed);
    });
    for i in 0..7 {
        let name = format!("__{}", i);
        spawn(add_client(
            name,
            IpAddr::V4(Ipv4Addr::LOCALHOST),
            25565,
            "127.0.0.1",
        ));
        sleep(Duration::from_secs(30)).await;
    }
    while !STOP.load(core::sync::atomic::Ordering::Relaxed) {
        sleep(Duration::from_millis(1)).await;
    }
}

async fn add_client(name: String, ip: IpAddr, port: u16, hostname: &str) -> tokio::io::Result<()> {
    let stream = TcpStream::connect(SocketAddr::new(ip, port)).await?;
    let _ = stream.set_nodelay(true);
    let mut client = Client {
        s: stream,
        b: Vec::new(),
    };
    client.write(&Intention {
        protocol_version: V32(minecraft_data::PROTOCOL_VERSION),
        host_name: Utf8(hostname),
        port,
        intention: ClientIntent::Login,
    });
    client.write(&Hello {
        name: Utf8(&name),
        profile_id: Uuid::nil(),
    });
    client.flush().await?;
    drop(name);
    let mut b = Vec::with_capacity(4096);
    let mut c = 0;
    let (_id, _name) = handle_login(&mut client, &mut b, &mut c).await?;
    handle_configuration(&mut client, &mut b, &mut c).await?;
    handle_game(&mut client, &mut b, &mut c).await?;
    Ok(())
}
