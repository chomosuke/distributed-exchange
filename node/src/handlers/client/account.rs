use super::{Req, UserID, Crud};
use crate::Global;
use lib::GResult;
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, .. }: &Req,
    global: &Arc<Global>,
) -> GResult<String> {
    match crud {
        Crud::Delete => {
            let mut state = global.state.write().await;
            let account = state
                .get_accounts_mut()
                .get(&user_id.id)
                .ok_or("Invalid account")?;
            let delete_status = account.write().await.delete().await ;
            match delete_status {
                Ok(()) => {
                    state.get_accounts_mut().remove(&user_id.id);
                    Ok("\"ok\"".to_string())
                }
                Err(_msg) => {
                    Ok("\"notEmpty\"".to_string())
                }
            }
        }
        _ => Err(Box::from(format!("Can not {crud:?} account."))),
    }
}
