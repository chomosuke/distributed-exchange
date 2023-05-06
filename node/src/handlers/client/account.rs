use super::{Crud, Req, UserID};
use crate::Global;
use lib::{GResult, lock::DeadLockDetect};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, .. }: Req,
    global: &Arc<Global>,
) -> GResult<String> {
    match crud {
        Crud::Delete => {
            let mut state = global.state.write().dl().await;
            let account = state.remove_account(user_id.id).ok_or("Invalid account")?;
            let delete_status = account.write().dl().await.delete().await;
            match delete_status {
                Ok(()) => {
                    Ok("\"ok\"".to_string())
                }
                Err(_msg) => Ok("\"notEmpty\"".to_string()),
            }
        }
        _ => Err(Box::from(format!("Can not {crud:?} account."))),
    }
}
