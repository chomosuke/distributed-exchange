use lib::read_writer::ReadWriter;
use serde::Deserialize;
use std::{error::Error, str::FromStr, sync::Arc};
use tokio::sync::mpsc;

use crate::{Global, Node, NodeID};

#[derive(Deserialize)]
pub struct FirstLine(pub NodeID);

impl FromStr for FirstLine {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Deserialize)]
struct State {
    id: NodeID,
    account_num: u64,
}

#[derive(Debug)]
pub enum Message {
    Order {
        t: String,
    },
}

pub async fn handler(
    FirstLine(id): FirstLine,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    let mut others = global.others.write().await;
    let addr = rw.peer_addr()?;

    // check if expecting
    if let Some(Node::DisConnected(expected_addr)) = others.get_mut(&id) {
        if expected_addr == &addr {
            Ok(())
        } else {
            Err(format!("Not expecting node {id} from {addr}"))
        }
    } else {
        Err(format!("Not expecting node {id}"))
    }?;

    let (sender, mut recver) = mpsc::unbounded_channel();

    others.insert(id, Node::Connected { sender });

    println!("Connected with Node {id} from addr {addr}");

    loop {

    }
}
