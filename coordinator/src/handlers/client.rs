use lib::{read_writer::ReadWriter, GResult, interfaces::UserID, lock::DeadLockDetect};
use std::{str::FromStr, sync::Arc};
use tokio::sync::oneshot;

use super::node::Message;
use crate::Global;

pub enum FirstLine {
    CAccount,
    FindNode(UserID),
}

impl FromStr for FirstLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(user_id) = serde_json::from_str(s) {
            Ok(FirstLine::FindNode(user_id))
        } else if s == "\"C Account\"" {
            Ok(FirstLine::CAccount)
        } else {
            Err("Did not match first line for client".into())
        }
    }
}

pub async fn handler(
    first_line: FirstLine,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> GResult<String> {
    match first_line {
        FirstLine::FindNode(user_id) => {
            let node_records = global.node_records.read().dl("34").await;
            let addr = node_records[user_id.node_id].address;

            rw.write_line(&addr.to_string()).await?;

            Ok(format!(
                "Found node {} for account {{ id: {}, node_id {} }}.",
                addr, user_id.id, user_id.node_id
            ))
        }
        FirstLine::CAccount => {
            let account_nums = global.account_nums.read().dl("45").await;
            let node_records = global.node_records.read().dl("46").await;

            let mut min_acc = 0;
            for i in 0..account_nums.len() {
                if account_nums[i] < account_nums[min_acc] {
                    min_acc = i;
                }
            }

            let (sender, recver) = oneshot::channel();

            node_records[min_acc]
                .sender
                .send(Message::CAccount(sender))?;

            let user_id = recver.await.map_err(|e| format!("user_id channel closed: {e}"))?;

            rw.write_line(&serde_json::to_string(&user_id)?).await?;

            Ok(format!(
                "Created account {{ id: {}, node_id {} }}.",
                user_id.id, user_id.node_id
            ))
        }
    }
}
