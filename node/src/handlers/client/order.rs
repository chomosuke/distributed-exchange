use super::{Crud, Req, UserID};
use crate::{
    matcher::Order,
    process_order::{process_order, OrderOrigin},
    Global,
};
use lib::{interfaces::OrderReq, lock::DeadLockDetect, GResult};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: Req,
    global: &Arc<Global>,
) -> GResult<String> {
    let state = global.state.read().dl("o15").await;
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
            let state = global.state.read().dl("o30").await;
            let account = state
                .get_accounts()
                .get(&user_id.id)
                .ok_or("Invalid account")?;
            account
                .write()
                .dl("o37")
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
                    OrderOrigin::Outgoing,
                    &global,
                )
                .await
                .expect("Process order failed");
            });
            Ok(r#""ok""#.to_owned())
        }
        Crud::Read => {
            let account = account.read().dl("o66").await;
            Ok(serde_json::to_string(&account.get_orders())?)
        }
        Crud::Delete => {
            let order: OrderReq = serde_json::from_value(value.ok_or("Bad value")?)?;
            let mut account = account.write().dl("o71").await;
            let quantity = account.deduct_order(order.clone()).await?;
            let OrderReq { order_type, ticker, price, .. } = order;
            global.matcher.write().dl("o74").await.deduct_order(Order {
                order_type, ticker, user_id: user_id.clone(), price, quantity,
            }).expect("Matcher account out of sync!");
            Ok(quantity.to_string())
        }
        _ => Err(Box::from(format!("Can not {crud:?} order."))),
    }
}
