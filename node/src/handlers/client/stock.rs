use super::{Crud, Req, UserID};
use crate::Global;
use lib::{GResult, lock::DeadLockDetect};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct CStockV {
    ticker_id: String,
    quantity: u64,
}

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
        Crud::Read => {
            let account = account.read().dl().await;
            Ok(serde_json::to_string(account.get_portfolio())?)
        }
        Crud::Create => {
            let account = &mut account.write().dl().await;
            let req: CStockV = serde_json::from_value(value.ok_or("Bad value".to_string())?)?;
            account.add_stock(req.ticker_id, req.quantity).await?;
            Ok("\"ok\"".to_string())
        }
        _ => Err(Box::from(format!("Can not {crud:?} market."))),
    }
}
