use super::Offer;
use crate::{handlers::node::OfferReply, Global};
use lib::{lock::DeadLockDetect, GResult, read_writer::ReadWriter};
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn handler(req: Value, rw: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let Offer { id, trade } = serde_json::from_value(req)?;
    let state = global.state.read().dl().await;
    let account = if trade.buyer_id.node_id == state.get_id() {
        state.get_accounts().get(&trade.buyer_id.id)
    } else {
        assert_eq!(
            trade.seller_id.node_id,
            state.get_id(),
            "Node recieved offer that it doesn't own"
        );
        state.get_accounts().get(&trade.seller_id.id)
    }
    .expect("Node recieved invalid UserID");
    let accepted = account
        .write()
        .dl()
        .await
        .process_incoming_offer(trade)
        .await?;
    rw.write_line(&serde_json::to_string(&json!({
        "type": "reply",
        "value": OfferReply {
            id,
            accepted,
        },
    }))?).await
}
