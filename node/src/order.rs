use crate::{handlers::node::Message, matcher::Order, Global, Node};
use lib::{lock::DeadLockDetect, GResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderUpdate {
    pub deduct: bool,

    #[serde(flatten)]
    pub order: Order,
}

pub fn add_order_to_matcher_and_process(order: Order, global: &Arc<Global>) {
    let global = Arc::clone(global);
    tokio::spawn(async move {
        _add_order_to_matcher_and_process(order, &global)
            .await
            .expect("Process order failed");
    });
}

/// add order to the matcher and process the matches
async fn _add_order_to_matcher_and_process(order: Order, global: &Arc<Global>) -> GResult<()> {
    let mut matcher = global.matcher.write().dl("pr12").await;
    let (remaining_order, matches, local_order_deducted) = matcher.add_order(order);

    for order in local_order_deducted {
        // need to broadcast
        broadcast_deduct_order(
            order,
            global.others.read().dl("o21").await.values().collect(),
        )
        .await?;
    }

    if remaining_order.user_id.node_id == global.state.read().dl("o24").await.get_id()
        && remaining_order.quantity > 0
    {
        // Send the order
        for (_, node) in global.others.read().dl("pr17").await.iter() {
            match node {
                crate::Node::DisConnected(addr) => todo!("{addr}"),
                crate::Node::Connected { sender } => sender.send(Message::Order(OrderUpdate {
                    deduct: false,
                    order: remaining_order.clone(),
                }))?,
            }
        }
    }

    // Process matches and register pending offer
    let offers = global
        .state
        .write()
        .dl("pr31")
        .await
        .process_matches(matches)
        .await?;

    // Now send the offers
    for (node_id, trade) in offers {
        match global
            .others
            .read()
            .dl("pr41")
            .await
            .get(&node_id)
            .expect("Bad node_id for trade offer")
        {
            crate::Node::DisConnected(addr) => todo!("{addr}"),
            crate::Node::Connected { sender } => sender.send(Message::Offer(trade))?,
        }
    }
    // Reply will be handled in handlers/node/offer_reply.rs
    Ok(())
}

// inform all matcher than order has been removed
pub async fn matcher_deduct_order(order: Order, global: &Arc<Global>) -> GResult<()> {
    global
        .matcher
        .write()
        .dl("o74")
        .await
        .deduct_order(order.clone());
    broadcast_deduct_order(order, global.others.read().await.values().collect()).await
}

pub async fn broadcast_deduct_order(order: Order, target_nodes: Vec<&Node>) -> GResult<()> {
    for node in target_nodes {
        match node {
            crate::Node::DisConnected(addr) => todo!("{addr}"),
            crate::Node::Connected { sender } => sender.send(Message::Order(OrderUpdate {
                deduct: true,
                order: order.clone(),
            }))?,
        }
    }
    Ok(())
}
