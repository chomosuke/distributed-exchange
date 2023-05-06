use super::get_value_type;
use crate::{Global, NodeID};
use lib::{read_writer::ReadWriter, GResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};

mod account;
mod balance;
// mod market;
// mod order;
// mod stock;

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct UserID {
    pub id: usize,
    pub node_id: NodeID,
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

#[derive(Debug)]
pub struct Req {
    crud: CRUD,
    target: Target,
    value: Option<Value>,
}

#[derive(Debug)]
enum CRUD {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug)]
enum Target {
    Balance,
    Stock,
    Market,
    Order,
    Account,
}

impl Req {
    fn from_str(s: &str) -> GResult<Req> {
        let (req_type, value) = get_value_type(s)?;

        let err: GResult<Req> = Err(Box::from(format!("Wrong type {}.", req_type)));

        let t = req_type.into_bytes();

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
            value,
        })
    }
}

pub async fn handler(
    FirstLine(user_id): FirstLine,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> GResult<String> {
    let state = global.state.read().await;
    if user_id.id >= state.get_accounts().len() || user_id.node_id != state.get_id() {
        return Err(Box::from(format!("Bad user_id: {user_id:?}")));
    }
    drop(state);
    loop {
        let line = rw.read_line().await?;
        if line == "bye" {
            return Ok(format!("Connection with user {user_id:?} terminated."));
        }
        let req = Req::from_str(&line)?;
        let res = match req.target {
            Target::Account => account::handler(&user_id, &req, &global).await?,
            Target::Balance => balance::handler(&user_id, &req, &global).await?,
            _ => return Err(Box::from("")),
            // Target::Market => market::handler(&user_id, &account, &req, &mut rw, &global).await?,
            // Target::Order => order::handler(&user_id, &account, &req, &mut rw, &global).await?,
            // Target::Stock => stock::handler(&user_id, &account, &req, &mut rw, &global).await?,
        };
        rw.write_line(&res);
        println!("repsonded request {req:?} from client {user_id:?} with {res:?}");
        if matches!(req.target, Target::Account) && matches!(req.crud, CRUD::Delete) {
            return Ok(format!(
                "Connection with user {user_id:?} terminated as account deleted."
            ));
        }
    }
}
