use super::{Req, UserID, CRUD};
use crate::Global;
use lib::GResult;
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: &Req,
    global: &Arc<Global>,
) -> GResult<String> {
    let state = global.state.read().await;
    let account = state
        .get_accounts()
        .get(&user_id.id)
        .ok_or("Invalid account")?;
    match crud {
        CRUD::Read => Ok(account.read().await.get_balance().to_string()),
        CRUD::Update => {
            let mut account = account.write().await;
            let current_balance = account.get_balance();
            let deducted =
                current_balance.min(value.as_ref().and_then(|v| v.as_u64()).ok_or("Bad value")?);
            account.set_balance(current_balance - deducted);
            Ok(deducted.to_string())
        }
        _ => Err(Box::from(format!("Can not {crud:?} account."))),
    }
}
