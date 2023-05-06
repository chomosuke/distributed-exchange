use super::{Crud, Req, UserID};
use crate::{handlers::node::Message, matcher::Order, Global};
use lib::{interfaces::OrderReq, lock::DeadLockDetect, GResult};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: Req,
    global: &Arc<Global>,
) -> GResult<String> {
    let state = global.state.read().dl().await;
    let account = state
        .get_accounts()
        .get(&user_id.id)
        .ok_or("Invalid account")?;
    match crud {
        Crud::Create => {
            let OrderReq {
                order_type,
                ticker,
                price,
                quantity,
            } = value
                .and_then(|v| serde_json::from_value(v).ok())
                .ok_or("Bad value")?;
            let state = global.state.read().dl().await;
            let account = state
                .get_accounts()
                .get(&user_id.id)
                .ok_or("Invalid account")?;
            account
                .write()
                .dl()
                .await
                .add_order(OrderReq {
                    order_type,
                    ticker: ticker.clone(),
                    quantity,
                    price,
                })
                .await?;
            let global = Arc::clone(global);
            let user_id = user_id.clone();
            tokio::spawn(async move {
                process_order(
                    Order {
                        order_type,
                        ticker,
                        user_id,
                        quantity,
                        price,
                    },
                    &global,
                )
                .await
                .expect("Process order failed");
            });
            Ok(r#""ok""#.to_owned())
        }
        Crud::Read => {
            let account = account.read().dl().await;
            Ok(serde_json::to_string(&account.get_orders())?)
        }
        // Crud::Delete => {}
        _ => Err(Box::from(format!("Can not {crud:?} order."))),
    }
}

pub async fn process_order(order: Order, global: &Arc<Global>) -> GResult<()> {
    let mut matcher = global.matcher.write().dl().await;
    let (remaining_order, matches) = matcher.add_order(order);

    if remaining_order.quantity > 0 {
        // Send the order
        for (_, node) in global.others.read().dl().await.iter() {
            match node {
                crate::Node::DisConnected(_) => todo!(),
                crate::Node::Connected { sender } => sender.send(Message::Order(remaining_order.clone()))?,
            }
        }
    }

    // Process matches and register pending offer
    let offers = global
        .state
        .write()
        .dl()
        .await
        .process_matches(matches)
        .await?;
    // Now send the offers
    for (node_id, trade) in offers {
        match global
            .others
            .read()
            .dl()
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
