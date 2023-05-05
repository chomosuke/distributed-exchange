use read_writer::ReadWriter;
use serde::Deserialize;
use std::{error::Error, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::oneshot::Sender;

use super::client::UserID;
use crate::{Global, NodeID};

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
    Joined(NodeID, SocketAddr),
    CAccount(Sender<UserID>),
}

pub async fn handler(
    first_line: FirstLine,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    let mut others = global.others.write().await;

    todo!()
}
