#![allow(clippy::new_without_default)]
mod handlers;

use crate::handlers::handler;
use lib::read_writer::ReadWriter;
use serde::Serialize;
use std::{net::SocketAddr, sync::Arc, error::Error};
use structopt::StructOpt;
use tokio::{
    net::TcpListener,
    sync::{mpsc::UnboundedSender, RwLock},
};

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    port: u16,
}

#[derive(Serialize)]
struct NodeRecord {
    address: SocketAddr,

    #[serde(skip)]
    sender: UnboundedSender<handlers::node::Message>,
}

pub struct Global {
    node_records: RwLock<Vec<NodeRecord>>,
    account_nums: RwLock<Vec<u64>>,
}

impl Global {
    pub fn new() -> Self {
        Self {
            node_records: RwLock::new(Vec::new()),
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
        let rw = match listener.accept().await {
            Ok((socket, _)) => ReadWriter::new(socket),
            Err(e) => {
                eprintln!("Error receiving connection from a new node: {e}");
                continue;
            }
        };
        let global = Arc::clone(&global);
        tokio::spawn(async {
            match handler(rw, global).await {
                Ok(msg) => println!("Connection terminated successfully: {msg}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        });
    }
}
