mod handlers;
mod matcher;

use crate::handlers::handler;
use read_writer::ReadWriter;
use serde::Deserialize;
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use structopt::StructOpt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::UnboundedSender, RwLock},
};

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,

    #[structopt(short, long)]
    addr: SocketAddr,
}

enum Server {
    DisConnected(SocketAddr),
    Connected {
        sender: UnboundedSender<handlers::Message>,
    },
    This,
}

struct Global {
    id: usize,
    others: RwLock<Vec<Server>>,
}

impl Global {
    pub fn new(id: usize, mut others: Vec<ServerRecord>) -> Self {
        others.sort_by_key(|sr| sr.id);
        let mut o = Vec::new();
        for other in others {
            if o.len() == id {
                o.push(Server::This);
            }
            assert!(o.len() == other.id, "Missing server with id {}", o.len());
            o.push(Server::DisConnected(other.address));
        }
        Self {
            id,
            others: RwLock::new(o),
        }
    }
}

#[derive(Deserialize)]
struct InitInfo {
    id: Option<usize>,
    others: Vec<ServerRecord>,
}
#[derive(Deserialize)]
struct ServerRecord {
    id: usize,
    address: SocketAddr,
}

#[tokio::main]
async fn main() {
    let Args { addr, coordinator } = Args::from_args();

    println!("Contacting coordinator on {}", coordinator);

    let listener: TcpListener = TcpListener::bind(addr).await.expect("Failed to bind");

    // Connect to coordinator
    let mut socket = TcpStream::connect(coordinator)
        .await
        .expect("Failed to connect coordinator");
    let mut coord_rw = ReadWriter::new(&mut socket);

    // send coordinator
    // TODO: Consider recovering node
    coord_rw
        .write_line(&serde_json::to_string(&json!({ "addr": &addr })).unwrap())
        .await
        .unwrap();

    // Get Id from coordinator
    let init_info: InitInfo =
        serde_json::from_str(&coord_rw.read_line().await.unwrap()).expect("Coordinator Error.");
    let id = init_info.id.expect("TODO");

    println!("Server Id: {}", id);

    let global = Arc::new(Global::new(id, init_info.others));

    loop {
        let mut socket = match listener.accept().await {
            Ok((socket, _)) => socket,
            Err(e) => {
                eprintln!("Error receiving connection from a new server: {e}");
                continue;
            }
        };
        let global = Arc::clone(&global);
        tokio::spawn(async move {
            match handler(ReadWriter::new(&mut socket), global).await {
                Ok(msg) => println!("Connection terminated successfully: {msg}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        });
    }
}
