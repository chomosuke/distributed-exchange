use lib::interfaces::UserID;
use lib::lock::DeadLockDetect;
use lib::{read_writer::ReadWriter, GResult};
use serde::Deserialize;
use serde_json::json;
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::sync::{mpsc, oneshot::Sender};

use crate::state::{NodeRecord, State};

#[derive(Deserialize)]
pub struct FirstLine {
    addr: SocketAddr,
    state: Option<NodeState>,
}

impl FromStr for FirstLine {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Deserialize)]
struct NodeState {
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
    mut rw: ReadWriter,
    state: Arc<State>,
) -> GResult<String> {
    let mut node_records = state.node_records.write().dl("41").await;
    let mut account_nums = state.account_nums.write().dl("42").await;
    let id = first_line
        .state
        .as_ref()
        .map(|s| s.id)
        .unwrap_or(node_records.get_records().len());
    let addr = first_line.addr;
    let rep = serde_json::to_string(&json!({
        "id": id,
        "others": node_records.get_records()
            .iter()
            .enumerate()
            .map(|(i, r)| json!({"id": i, "addr": r.address}))
            .collect::<Vec<_>>(),
    }))?;
    // reply with ID and all other servers.
    rw.write_line(&rep).await?;

    let (sender, mut recver) = mpsc::unbounded_channel();
    if let Some(state) = first_line.state {
        node_records
            .set_record(
                id,
                NodeRecord {
                    address: addr,
                    sender: Some(sender),
                },
            )
            .await;
        account_nums.set_num(id, state.account_num).await;
    } else {
        node_records
            .add_record(NodeRecord {
                address: first_line.addr,
                sender: Some(sender),
            })
            .await;
        account_nums.add_num(0).await;
    }

    let line = rw.read_line().await?;
    if line != "\"ok\"" {
        panic!("Node at {addr} replied with {line} instead of \"ok\"",);
    }

    // inform all other nodes that this node has joined
    let records = node_records.get_records();
    for node in records.iter().take(records.len() - 1) {
        node.sender
            .as_ref()
            .expect("TODO")
            .send(Message::Joined(id, addr))?;
    }

    // release the write lock
    drop(node_records);
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
                }))?)
                .await?;
            }
            Message::CAccount(sender) => {
                rw.write_line(&serde_json::to_string(&json!({
                    "type": "C account",
                }))?)
                .await?;

                let line = rw.read_line().await?;
                println!("{id} {line}");
                sender
                    .send(serde_json::from_str(&line)?)
                    .map_err(|_| line.clone())?;
            }
        }
    }
}
