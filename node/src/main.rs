mod accounts;
mod handlers;
mod matcher;

use crate::{handlers::handler, accounts::Accounts};
use lib::read_writer::ReadWriter;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
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

    #[structopt(short, long)]
    presistant_dir: String,
}

enum Node {
    DisConnected(SocketAddr),
    Connected {
        sender: UnboundedSender<handlers::node::Message>,
    },
}

pub type NodeID = usize;

pub struct Global {
    id: NodeID,
    others: RwLock<HashMap<NodeID, Node>>,
}

impl Global {
    pub fn new(id: NodeID, others: Vec<NodeRecord>) -> Self {
        Self {
            id,
            others: RwLock::new(
                others
                    .iter()
                    .map(|sr| (sr.id, Node::DisConnected(sr.address)))
                    .collect(),
            ),
        }
    }
}

#[derive(Deserialize)]
struct InitInfo {
    id: Option<NodeID>,
    others: Vec<NodeRecord>,
}
#[derive(Deserialize)]
pub struct NodeRecord {
    id: NodeID,
    address: SocketAddr,
}

#[tokio::main]
async fn main() {
    let Args {
        addr,
        coordinator,
        presistant_dir,
    } = Args::from_args();

    // apart from account file, everything is stored in the 'state' file
    let mut accounts = Accounts::restore(presistant_dir);

    println!("Contacting coordinator on {}", coordinator);

    let listener: TcpListener = TcpListener::bind(addr).await.expect("Failed to bind");

    // Connect to coordinator
    let mut coord_rw = ReadWriter::new(
        TcpStream::connect(coordinator)
            .await
            .expect("Failed to connect coordinator"),
    );

    // send coordinator
    coord_rw
        .write_line(&serde_json::to_string(&json!({ "addr": &addr })).unwrap())
        .await
        .unwrap();

    // Get Id from coordinator
    let init_info: InitInfo =
        serde_json::from_str(&coord_rw.read_line().await.unwrap()).expect("Coordinator Error.");
    let id = init_info.id.expect("TODO");

    println!("Node Id: {}", id);

    coord_rw.write_line("ok").await.expect("Write failed");

    let global = Arc::new(Global::new(id, init_info.others));

    {
        // spawn task to communicate with coordinator
        let global = Arc::clone(&global);
        tokio::spawn(async {
            match handlers::coordinator::handler(coord_rw, global).await {
                Ok(msg) => println!("Connection terminated successfully: {msg}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        });
    }

    loop {
        let rw = match listener.accept().await {
            Ok((socket, _)) => ReadWriter::new(socket),
            Err(e) => {
                eprintln!("Error receiving connection: {e}");
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
