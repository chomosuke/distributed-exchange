use super::OfferReply;
use crate::{order::add_order_to_matcher_and_process, Global};
use lib::{lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::sync::Arc;

pub async fn handler(req: Value, _: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let OfferReply { id, accepted } = serde_json::from_value(req)?;
    let mut state = global.state.write().dl("ofrp9").await;
    if accepted {
        state.commit_pending(id).await?;
    } else {
        let order = state.abort_pending(id).await?;
        drop(state);
        add_order_to_matcher_and_process(order, global);
    }
    Ok(())
}
