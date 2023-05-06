use crate::Global;
use lib::{read_writer::ReadWriter, GResult};
use serde_json::json;
use std::sync::Arc;
use super::Offer;

pub async fn handler(offer: Offer, rw: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    rw.write_line(&serde_json::to_string(&json!({
        "type": "offer",
        "value": offer,
    }))?).await
}
