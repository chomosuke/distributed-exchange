use super::Offer;
use crate::{handlers::node::OfferReply, order::broadcast_deduct_order, Global};
use lib::{lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde_json::{json, Value};
use std::sync::Arc;

/// recieved a trade offer
pub async fn handler(req: Value, rw: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let Offer { id, trade } = serde_json::from_value(req)?;
    let state = global.state.read().dl("of9").await;
    let (account, other_node) = if trade.buyer_id.node_id == state.get_id() {
        (
            state
                .get_accounts()
                .get(&trade.buyer_id.id)
                .expect("Node recieved invalid UserID"),
            trade.seller_id.node_id,
        )
    } else {
        assert_eq!(
            trade.seller_id.node_id,
            state.get_id(),
            "Node recieved offer that it doesn't own"
        );
        (
            state
                .get_accounts()
                .get(&trade.seller_id.id)
                .expect("Node recieved invalid UserID"),
            trade.buyer_id.node_id,
        )
    };
    let order_deducted = account
        .write()
        .dl("of23")
        .await
        .process_incoming_offer(trade)
        .await?;

    let accepted = order_deducted.is_some();

    if let Some(order) = order_deducted {
        // update the matcher to remove the order
        broadcast_deduct_order(
            order,
            global
                .others
                .read()
                .await
                .iter()
                .filter_map(|(node_id, n)| {
                    if *node_id != other_node {
                        Some(n)
                    } else {
                        None
                    }
                })
                .collect(),
        )
        .await?;
    }

    rw.write_line(&serde_json::to_string(&json!({
        "type": "reply",
        "value": OfferReply {
            id,
            accepted,
        },
    }))?)
    .await
}
