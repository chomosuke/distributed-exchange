use super::OfferReply;
use crate::Global;
use lib::{lock::DeadLockDetect, GResult, read_writer::ReadWriter};
use serde_json::Value;
use std::sync::Arc;

pub async fn handler(req: Value, _: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let OfferReply { id, accepted } = serde_json::from_value(req)?;
    let mut state = global.state.write().dl().await;
    if accepted {
        state.commit_pending(id).await
    } else {
        state.abort_pending(id).await
    }
}
