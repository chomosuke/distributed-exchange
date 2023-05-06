mod handlers;
mod matcher;
mod state;

use crate::{handlers::handler, state::State};
use lib::read_writer::ReadWriter;
use serde::Deserialize;
use serde_json::json;
use state::NodeID;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, error::Error};
use structopt::StructOpt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::UnboundedSender, RwLock},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,

    #[structopt(short, long)]
    addr: SocketAddr,

    #[structopt(short, long)]
    persistent_dir: String,
}

enum Node {
    DisConnected(SocketAddr),
    Connected {
        sender: UnboundedSender<handlers::node::Message>,
    },
}

pub struct Global {
    state: RwLock<State>,
    others: RwLock<HashMap<NodeID, Node>>,
}

impl Global {
    pub fn new(state: State, others: Vec<NodeRecord>) -> Self {
        Self {
            state: RwLock::new(state),
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
        persistent_dir,
    } = Args::from_args();

    let state = State::restore(persistent_dir.clone()).await;

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
        .write_line(
            &serde_json::to_string(&json!({
                "addr": &addr,
                "id": state.as_ref().map(|s| s.get_id())
            }))
            .unwrap(),
        )
        .await
        .unwrap();

    // Get Id from coordinator
    let init_info: InitInfo =
        serde_json::from_str(&coord_rw.read_line().await.unwrap()).expect("Coordinator Error.");

    let state = state.unwrap_or_else(|| {
        State::new(
            init_info.id.expect("Expected NodeID from coordinator"),
            persistent_dir.clone(),
        )
    });

    println!("Node Id: {}", state.get_id());

    coord_rw.write_line("ok").await.expect("Write failed");

    let global = Arc::new(Global::new(state, init_info.others));

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
