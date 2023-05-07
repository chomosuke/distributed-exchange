use crate::Global;
use lib::{read_writer::ReadWriter, GResult};
use serde_json::{Map, Value};
use std::{str::FromStr, sync::Arc};

pub mod client;
pub mod coordinator;
pub mod node;

pub async fn handler(mut rw: ReadWriter, global: Arc<Global>) -> GResult<String> {
    let first_line = rw.read_line().await?;

    if let Ok(first_line) = client::FirstLine::from_str(&first_line) {
        client::handler(first_line, rw, global).await
    } else if let Ok(first_line) = node::FirstLine::from_str(&first_line) {
        tokio::spawn(async move {
            match node::handler(first_line, rw, global).await {
                Ok(msg) => println!("Connection terminated with node: {msg}"),
                Err(e) => eprintln!("Error: {e}"),
            }
        });
        Ok("Started communicating with node".to_owned())
    } else {
        Err(format!("{first_line} is not a valid request").into())
    }
}

fn get_value_type(s: &str) -> GResult<(String, Option<Value>)> {
    let mut obj = serde_json::from_str(s)
        .ok()
        .and_then(|v: Value| serde_json::from_value::<Map<_, _>>(v).ok())
        .ok_or("Not valid json object")?;

    Ok((
        obj.get("type")
            .and_then(|v: &Value| v.as_str())
            .ok_or("Doesn't have member type")?
            .to_owned(),
        obj.remove("value"),
    ))
}
