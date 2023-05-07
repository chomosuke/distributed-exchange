use super::{Crud, Req, UserID};
use crate::Global;
use lib::{lock::DeadLockDetect, GResult};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: Req,
    global: &Arc<Global>,
) -> GResult<String> {
    let state = global.state.read().dl("11").await;
    let account = state
        .get_accounts()
        .get(&user_id.id)
        .ok_or("Invalid account")?;
    match crud {
        Crud::Read => Ok(account.read().dl("17").await.get_balance().to_string()),
        Crud::Update => {
            let mut account = account.write().dl("19").await;
            let new_balance = value.as_ref().and_then(|v| v.as_u64()).ok_or("Bad value")?;
            let empty = account.set_balance(new_balance).await?;
            if empty {
                Ok("\"ok\"".to_string())
            } else {
                Ok("\"notEmpty\"".to_string())
            }
        }
        _ => Err(Box::from(format!("Can not {crud:?} balance."))),
    }
}
