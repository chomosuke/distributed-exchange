use super::PendingOffer;
use crate::Global;
use lib::{read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::sync::Arc;

pub async fn handler(
    req: &Value,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
    pending_offer: &mut PendingOffer,
) -> GResult<String> {
    todo!()
}
