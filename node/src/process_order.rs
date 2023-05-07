use crate::{handlers::node::Message, matcher::Order, Global};
use lib::{lock::DeadLockDetect, GResult};
use std::sync::Arc;

#[derive(PartialEq)]
pub enum OrderOrigin {
    Incoming,
    Outgoing,
}

/// add order to the matcher and process the matches
pub async fn process_order(order: Order, origin: OrderOrigin, global: &Arc<Global>) -> GResult<()> {
    let mut matcher = global.matcher.write().dl("pr12").await;
    let (remaining_order, matches) = matcher.add_order(order);

    if remaining_order.quantity > 0 && origin == OrderOrigin::Outgoing {
        // Send the order
        for (_, node) in global.others.read().dl("pr17").await.iter() {
            match node {
                crate::Node::DisConnected(_) => todo!(),
                crate::Node::Connected { sender } => {
                    sender.send(Message::Order(remaining_order.clone()))?
                }
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
            crate::Node::DisConnected(_) => todo!(),
            crate::Node::Connected { sender } => sender.send(Message::Offer(trade))?,
        }
    }
    // Reply will be handled in handlers/node/offer_reply.rs
    Ok(())
}
