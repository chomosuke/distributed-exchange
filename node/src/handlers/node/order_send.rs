use crate::{Global, matcher::Order};
use lib::{read_writer::ReadWriter, GResult};
use serde_json::json;
use std::sync::Arc;

pub async fn handler(order: Order, rw: &mut ReadWriter, _global: &Arc<Global>) -> GResult<()> {
    rw.write_line(&serde_json::to_string(&json!({
        "type": "order",
        "value": order,
    }))?).await
}

