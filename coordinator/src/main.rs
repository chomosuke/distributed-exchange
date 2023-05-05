#![allow(clippy::new_without_default)]
use std::{net::SocketAddr, sync::Arc};

use read_writer::ReadWriter;
use serde::Serialize;
use structopt::StructOpt;
use tokio::{
    net::TcpListener,
    sync::{mpsc::UnboundedSender, RwLock},
};

use crate::handlers::handler;

mod handlers;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    port: u16,
}

#[derive(Serialize)]
struct ServerRecord {
    address: SocketAddr,

    #[serde(skip_serializing)]
    sender: UnboundedSender<handlers::node::Message>,
}

pub struct Global {
    server_records: RwLock<Vec<ServerRecord>>,
    account_nums: RwLock<Vec<u64>>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            server_records: RwLock::new(Vec::new()),
            account_nums: RwLock::new(Vec::new()),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let ip_port = format!("127.0.0.1:{}", args.port);
    println!("Starting coordinator on {ip_port}");
    let listener: TcpListener = TcpListener::bind(ip_port).await.expect("Failed to bind");

    let global: Arc<Global> = Arc::new(Global::new());

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
