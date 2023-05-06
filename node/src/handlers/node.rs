mod offer_recv;
mod offer_replied;
mod offer_send;
mod order;

use crate::{handlers::get_value_type, matcher::Trade, Global, Node, NodeID};
use lib::{lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};
use tokio::{select, sync::mpsc};

#[derive(Deserialize)]
pub struct FirstLine(pub NodeID);

impl FromStr for FirstLine {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Debug)]
pub enum Message {
    Offer(Offer),
}

pub type TradeID = usize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer {
    pub id: TradeID,

    #[serde(flatten)]
    pub trade: Trade,
}

#[derive(Serialize, Deserialize)]
struct OfferReply {
    id: TradeID,
    accepted: bool,
}

pub async fn handler(
    FirstLine(id): FirstLine,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> GResult<String> {
    let mut others = global.others.write().dl().await;
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
        let res = select! {
            msg = recver.recv() => {
                let msg = msg.ok_or(format!("Channel for node {addr} closed!"))?;
                match msg {
                    Message::Offer(offer) => offer_send::handler(offer, &mut rw, &global).await?,
                }
            },
            line = rw.read_line() => {
                let line = line?;
                let (req_type, value) = get_value_type(&line)?;
                let value = value.ok_or("No value for request")?;
                match req_type.as_str() {
                    // "order" => order::handler(&value, &mut rw, &global).await?,
                    "offer" => offer_recv::handler(value, &global).await?,
                    // "reply" => offer_reply::handler(&value, &mut rw, &global, &mut pending_offer).await?,
                    req_type => return Err(Box::from(format!("Wrong type {}.", req_type))),
                }
            },
        };
        rw.write_line(&res).await?;
        println!("Sent {res} to node {id}");
    }
}
