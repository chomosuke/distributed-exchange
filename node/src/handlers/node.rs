use read_writer::ReadWriter;
use serde::Deserialize;
use serde_json::json;
use std::{error::Error, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::{mpsc, oneshot::Sender};

use super::client::UserID;
use crate::{Global, ServerRecord};

#[derive(Deserialize)]
pub struct FirstLine {
    addr: SocketAddr,
    state: Option<State>,
}

impl FromStr for FirstLine {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Deserialize)]
struct State {
    id: usize,
    account_num: u64,
}

#[derive(Debug)]
pub enum Message {
    Joined(usize, SocketAddr),
    CAccount(Sender<UserID>),
}

pub async fn handler(
    first_line: FirstLine,
    mut rw: ReadWriter<'_>,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    let mut server_records = global.server_records.write().await;
    let mut account_nums = global.account_nums.write().await;
    let id = first_line
        .state
        .as_ref()
        .map(|s| s.id)
        .unwrap_or(server_records.len());
    let addr = first_line.addr;
    let rep = serde_json::to_string(&json!({
        "id": id,
        "others": server_records
            .iter()
            .enumerate()
            .map(|(i, r)| json!({"id": i, "addr": r.address}))
            .collect::<Vec<_>>(),
    }))?;
    // reply with ID and all other servers.
    rw.write_line(&rep).await?;

    let (sender, mut recver) = mpsc::unbounded_channel();
    if let Some(state) = first_line.state {
        server_records[id].address = addr;
        account_nums[id] = state.account_num;
        server_records[id].sender = sender;
    } else {
        server_records.push(ServerRecord {
            address: first_line.addr,
            sender,
        });
        account_nums.push(0);
    }

    let line = rw.read_line().await?;
    if line != "ok" {
        panic!("Node at {addr} replied with {line} instead of \"ok\"",);
    }

    // inform all other nodes that this node has joined
    for i in 0..(server_records.len() - 1) {
        let server_record = &server_records[i];
        server_record.sender.send(Message::Joined(id, addr))?;
    }

    // release the write lock
    drop(server_records);
    drop(account_nums);

    loop {
        let msg = recver
            .recv()
            .await
            .ok_or(format!("Channel for node {addr} is closed!"))?;

        match msg {
            Message::Joined(id, addr) => {
                rw.write_line(&serde_json::to_string(&json!({
                    "type": "joined",
                    "id": id,
                    "addr": addr,
                }))?).await?;
            }
            Message::CAccount(sender) => {
                rw.write_line(&serde_json::to_string(&json!({
                    "type": "C account",
                }))?).await?;

                let line = rw.read_line().await?;
                sender.send(serde_json::from_str(&line)?).map_err(|_| line.clone())?;
            },
        }
    }
}