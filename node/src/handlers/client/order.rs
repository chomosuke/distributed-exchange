use super::{Crud, Req, UserID};
use crate::{Global, state};
use lib::{
    interfaces::{CentCount, Quantity, Ticker, OrderReq},
    GResult,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: Req,
    global: &Arc<Global>,
) -> GResult<String> {
    let state = global.state.read().await;
    let account = state
        .get_accounts()
        .get(&user_id.id)
        .ok_or("Invalid account")?;
    match crud {
        Crud::Create => {
            let req: OrderReq = value.and_then(|v| serde_json::from_value(v).ok()).ok_or("Bad value")?;
            let state = global.state.read().await;
            let account = state
                .get_accounts()
                .get(&user_id.id)
                .ok_or("Invalid account")?;
            account.write().await.add_order(state::Order {order_type:  });
            let matcher = global.matcher.write().await;
            matcher.add_order();
        }
        Crud::Read => {
            let account = account.read().await;
            Ok(serde_json::to_string(&account.get_orders())?)
        }
        // Crud::Delete => {}
        _ => Err(Box::from(format!("Can not {crud:?} order."))),
    }
}
