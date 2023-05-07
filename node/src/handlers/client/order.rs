use super::{Crud, Req, UserID};
use crate::{
    matcher::Order,
    order::{add_order_to_matcher_and_process, matcher_deduct_order},
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

            let enough = account
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
            if !enough {
                return Ok(r#""notEnough""#.to_owned());
            }

            let global = Arc::clone(global);
            let user_id = *user_id;
            add_order_to_matcher_and_process(
                Order {
                    order_type,
                    ticker,
                    user_id,
                    quantity,
                    price,
                },
                &global,
            );
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
            let OrderReq {
                order_type,
                ticker,
                price,
                ..
            } = order;

            matcher_deduct_order(
                Order {
                    order_type,
                    ticker,
                    user_id: *user_id,
                    price,
                    quantity,
                },
                global,
            )
            .await?;

            Ok(quantity.to_string())
        }
        _ => Err(Box::from(format!("Can not {crud:?} order."))),
    }
}
