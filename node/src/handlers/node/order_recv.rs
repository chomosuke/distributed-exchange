use crate::{
    matcher::Order,
    process_order::{process_order, OrderOrigin},
    Global,
};
use lib::{read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::sync::Arc;

pub async fn handler(req: Value, _rw: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let order: Order = serde_json::from_value(req)?;
    let global = Arc::clone(global);
    tokio::spawn(async move {
        process_order(order, OrderOrigin::Incoming, &global)
            .await
            .expect("Process order failed");
    });
    Ok(())
}
