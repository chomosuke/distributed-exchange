use super::get_value_type;
use crate::Global;
use lib::{interfaces::UserID, lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::{str::FromStr, sync::Arc};

mod account;
mod balance;
mod market;
mod order;
mod stock;

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
    crud: Crud,
    target: Target,
    value: Option<Value>,
}

#[derive(Debug, Clone, Copy)]
enum Crud {
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy)]
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
            b'C' => Crud::Create,
            b'R' => Crud::Read,
            b'U' => Crud::Update,
            b'D' => Crud::Delete,
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
    let state = global.state.read().dl("c95").await;
    if user_id.id >= state.get_accounts().len() || user_id.node_id != state.get_id() {
        return Err(Box::from(format!("Bad user_id: {user_id:?}")));
    }
    drop(state);
    loop {
        let line = rw.read_line().await?;
        if line == "\"bye\"" {
            return Ok(format!("Connection with user {user_id:?} terminated."));
        }
        let req = Req::from_str(&line)?;
        let target = req.target;
        let crud = req.crud;
        let res = match req.target {
            Target::Account => account::handler(&user_id, req, &global).await?,
            Target::Balance => balance::handler(&user_id, req, &global).await?,
            Target::Market => market::handler(req, &global).await?,
            Target::Order => order::handler(&user_id, req, &global).await?,
            Target::Stock => stock::handler(&user_id, req, &global).await?,
        };
        rw.write_line(&res).await?;
        if matches!(target, Target::Account) && matches!(crud, Crud::Delete) && res == "\"ok\"" {
            return Ok(format!(
                "Connection with user {user_id:?} terminated as account deleted."
            ));
        }
    }
}
