mod offer_recv;
mod offer_reply;
mod offer_send;
mod order;

use crate::{handlers::get_value_type, matcher::Trade, Global, Node, NodeID};
use lib::{read_writer::ReadWriter, GResult};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::{select, sync::mpsc};

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
    Offer(Trade),
}

pub type TradeID = usize;

#[derive(Serialize, Deserialize)]
struct Offer {
    id: TradeID,

    #[serde(flatten)]
    trade: Trade,
}

#[derive(Serialize, Deserialize)]
struct OfferReply {
    id: TradeID,
    accepted: bool,
}

pub struct PendingOffer {
    pending: HashMap<TradeID, Trade>,
    next_trade_id: TradeID,
}

pub async fn handler(
    FirstLine(id): FirstLine,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> GResult<String> {
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
        select! {
            msg = recver.recv() => {
                let msg = msg.ok_or(format!("Channel for node {addr} closed!"))?;
                let result = match msg {
                    // Message::Matched(trade) => offer_send::handler(trade, &mut rw, &global, &mut pending_offer).await?,
                    _ => return Err(Box::from("")),
                };
            },
            line = rw.read_line() => {
                let line = line?;
                let (req_type, value) = get_value_type(&line)?;
                let value = value.ok_or("No value for request")?;
                let result = match req_type.as_str() {
                    // "order" => order::handler(&value, &mut rw, &global).await?,
                    // "offer" => offer_recv::handler(&value, &mut rw, &global).await?,
                    // "reply" => offer_reply::handler(&value, &mut rw, &global, &mut pending_offer).await?,
                    req_type => return Err(Box::from(format!("Wrong type {}.", req_type))),
                };
            },
        }
    }
}
