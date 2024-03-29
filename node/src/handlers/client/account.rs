use super::{Crud, Req, UserID};
use crate::Global;
use lib::{lock::DeadLockDetect, GResult};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, .. }: Req,
    global: &Arc<Global>,
) -> GResult<String> {
    match crud {
        Crud::Delete => {
            let state = global.state.read().dl("13").await;
            let account = state
                .get_accounts()
                .get(&user_id.id)
                .ok_or("Invalid account")?;
            let delete_status = account.write().dl("15").await.delete().await;
            match delete_status {
                Ok(()) => {
                    global.state.write().dl("13").await.remove_account(user_id.id);
                    Ok("\"ok\"".to_string())
                }
                Err(_msg) => Ok("\"notEmpty\"".to_string()),
            }
        }
        _ => Err(Box::from(format!("Can not {crud:?} account."))),
    }
}
