use super::{Req, UserID, Crud};
use crate::Global;
use lib::{GResult, lock::DeadLockDetect};
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
        Crud::Read => Ok(account.read().dl().await.get_balance().to_string()),
        Crud::Update => {
            let mut account = account.write().dl().await;
            let new_balance = value.as_ref().and_then(|v| v.as_u64()).ok_or("Bad value")?;
            account.set_balance(new_balance).await?;
            Ok("\"ok\"".to_string())
        }
        _ => Err(Box::from(format!("Can not {crud:?} balance."))),
    }
}
