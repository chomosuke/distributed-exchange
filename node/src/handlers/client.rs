use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{error::Error, str::FromStr, sync::Arc};
use tokio::sync::oneshot;

use super::{Message, ReadWriter};
use crate::Global;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserID {
    id: u64,
    node_id: usize,
}

pub struct FirstLine {
    crud: CRUD,
    target: Target,
    value: Option<Value>,
}

enum CRUD {
    Create,
    Read,
    Update,
    Delete,
}

enum Target {
    Balance,
    Stock,
    Market,
    Order,
    Account,
}

impl FromStr for FirstLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let obj = serde_json::from_str(s)
            .ok()
            .and_then(|v: Value| v.as_object())
            .ok_or("Not valid json object or ")?;

        let t = obj
            .get("type")
            .and_then(|v: &Value| v.as_str())
            .ok_or("Doesn't have member type")?
            .as_bytes();

        let err = Err("Did not match first line for client".into());

        if t.len() < 3 {
            return err;
        }

        let crud = match t[0] {
            b'C' => CRUD::Create,
            b'R' => CRUD::Read,
            b'U' => CRUD::Update,
            b'D' => CRUD::Delete,
            _ => return err,
        };

        if t[1] != b' ' {
            return err;
        }

        let target = match &String::from_utf8(t[2..].to_vec()).unwrap()[..] {
            "balance" => Target::Balance,
            "stock" => Target::Stock,
            "market" => Target::Market,
            "order" => Target::Order,
            "account" => Target::Account,
            _ => return err,
        };

        Ok(FirstLine {
            crud,
            target,
            value: obj.get("value").cloned(),
        })
    }
}

pub async fn handler(
    first_line: FirstLine,
    mut rw: ReadWriter<'_>,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    match first_line {
        FirstLine::FindNode(user_id) => {
            let server_records = global.server_records.read().await;
            let addr = server_records[user_id.node_id].address;

            rw.write_line(&addr.to_string()).await?;

            Ok(format!(
                "Found node {} for account {{ id: {}, node_id {} }}.",
                addr, user_id.id, user_id.node_id
            ))
        }
        FirstLine::CAccount => {
            let account_nums = global.account_nums.read().await;
            let server_records = global.server_records.read().await;

            let mut min_acc = 0;
            for i in 0..account_nums.len() {
                if account_nums[i] < account_nums[min_acc] {
                    min_acc = i;
                }
            }

            let (sender, recver) = oneshot::channel();

            server_records[min_acc]
                .sender
                .send(Message::CAccount(sender))?;

            let user_id = recver.await?;

            rw.write_line(&serde_json::to_string(&user_id)?).await?;

            Ok(format!(
                "Created account {{ id: {}, node_id {} }}.",
                user_id.id, user_id.node_id
            ))
        }
    }
}
