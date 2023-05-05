use read_writer::ReadWriter;
use serde_json::{Map, Value};
use std::{error::Error, str::FromStr, sync::Arc};

use crate::Global;

pub mod client;
pub mod coordinator;
pub mod node;

pub async fn handler(mut rw: ReadWriter, global: Arc<Global>) -> Result<String, Box<dyn Error>> {
    let first_line = rw.read_line().await?;

    if let Ok(first_line) = client::FirstLine::from_str(&first_line) {
        client::handler(first_line, rw, global).await
    } else if let Ok(first_line) = node::FirstLine::from_str(&first_line) {
        node::handler(first_line, rw, global).await
    } else {
        Err(format!("{first_line} is not a valid request").into())
    }
}

fn get_type(s: &str) -> Result<String, Box<dyn Error>> {
    let obj = serde_json::from_str(s)
        .ok()
        .and_then(|v: Value| serde_json::from_value::<Map<_, _>>(v).ok())
        .ok_or("Not valid json object")?;

    Ok(obj
        .get("type")
        .and_then(|v: &Value| v.as_str())
        .ok_or("Doesn't have member type")?
        .to_owned())
}
