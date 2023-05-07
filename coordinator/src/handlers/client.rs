use lib::{interfaces::UserID, lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use std::{str::FromStr, sync::Arc};
use tokio::sync::oneshot;

use super::node::Message;
use crate::State;

pub enum FirstLine {
    CAccount,
    FindNode(UserID),
}

impl FromStr for FirstLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(user_id) = serde_json::from_str(s) {
            Ok(FirstLine::FindNode(user_id))
        } else if s == "\"C account\"" {
            Ok(FirstLine::CAccount)
        } else {
            Err("Did not match first line for client".into())
        }
    }
}

pub async fn handler(
    first_line: FirstLine,
    mut rw: ReadWriter,
    state: Arc<State>,
) -> GResult<String> {
    let node_records = state.node_records.read().dl("cl32").await;
    let node_records = node_records.get_records();
    match first_line {
        FirstLine::FindNode(user_id) => {
            let addr = node_records[user_id.node_id].address;

            rw.write_line(&addr.to_string()).await?;

            Ok(format!(
                "Found node {} for account {{ id: {}, node_id {} }}.",
                addr, user_id.id, user_id.node_id
            ))
        }
        FirstLine::CAccount => {
            let mut account_nums = state.account_nums.write().dl("45").await;

            let mut min_acc = 0;
            let a_nums = account_nums.get_nums();
            for i in 0..a_nums.len() {
                if a_nums[i] < a_nums[min_acc] {
                    min_acc = i;
                }
            }
            let min_num = a_nums[min_acc];
            account_nums.set_num(min_acc, min_num + 1).await;

            let (sender, recver) = oneshot::channel();

            node_records[min_acc]
                .sender
                .as_ref()
                .expect("TODO")
                .send(Message::CAccount(sender))?;

            let user_id = recver
                .await
                .map_err(|e| format!("user_id channel closed: {e}"))?;

            rw.write_line(&serde_json::to_string(&user_id)?).await?;

            Ok(format!(
                "Created account {{ id: {}, node_id {} }}.",
                user_id.id, user_id.node_id
            ))
        }
    }
}
