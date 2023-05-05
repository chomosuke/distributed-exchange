use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{error::Error, str::FromStr, sync::Arc};

use super::ReadWriter;
use crate::{Global, NodeID};

mod account;
mod balance;
mod market;
mod order;
mod stock;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserID {
    id: u64,
    node_id: NodeID,
}

pub struct FirstLine(UserID);

impl FromStr for FirstLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FirstLine(serde_json::from_str(s).map_err(|_| {
            "Did not match first line for client".to_owned()
        })?))
    }
}

struct Req {
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

impl Req {
    fn from_str(s: &str) -> Result<Req, Box<dyn Error>> {
        let obj = serde_json::from_str(s)
            .ok()
            .and_then(|v: Value| v.as_object())
            .ok_or("Not valid json object or ")?;

        let t = obj
            .get("type")
            .and_then(|v: &Value| v.as_str())
            .ok_or("Doesn't have member type")?
            .as_bytes();

        let err: Result<Req, Box<dyn Error>> = Err(Box::from(format!(
            "wrong type {}",
            String::from_utf8(t.to_vec()).unwrap()
        )));

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

        Ok(Req {
            crud,
            target,
            value: obj.remove("value"),
        })
    }
}

pub async fn handler(
    FirstLine(user_id): FirstLine,
    mut rw: ReadWriter<'_>,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    let req = Req::from_str(&rw.read_line().await?)?;
    match req.target {
        Target::Account => account::handler(req, rw, global).await,
        Target::Balance => balance::handler(req, rw, global).await,
        Target::Market => market::handler(req, rw, global).await,
        Target::Order => order::handler(req, rw, global).await,
        Target::Stock => stock::handler(req, rw, global).await,
    }
}
