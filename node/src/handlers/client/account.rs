use super::{Req, UserID, CRUD};
use crate::Global;
use lib::{read_writer::ReadWriter, GResult};
use std::sync::Arc;

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: &Req,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
) -> GResult<String> {
    match crud {
        CRUD::Delete => {
            let mut state = global.state.write().await;
            let account = state
                .get_accounts_mut()
                .get(&user_id.id)
                .ok_or("Invalid account")?;
            let delete_status = account.write().await.delete().await ;
            match delete_status {
                Ok(()) => {
                    state.get_accounts_mut().remove(&user_id.id);
                    rw.write_line("\"Ok\"").await?;
                    Ok(format!("Successfully deleted account {user_id:?}"))
                }
                Err(msg) => {
                    rw.write_line("\"NotEmpty\"");
                    Ok(msg)
                }
            }
        }
        _ => Err(Box::from(format!("Can not {crud:?} account."))),
    }
}
