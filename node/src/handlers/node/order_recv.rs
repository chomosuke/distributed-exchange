use crate::{
    order::{add_order_to_matcher_and_process, OrderUpdate},
    Global,
};
use lib::{lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::sync::Arc;

pub async fn handler(req: Value, _rw: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let OrderUpdate { deduct, order } = serde_json::from_value(req)?;
    if deduct {
        println!("deduct order recieved {:?}", order.clone());
        global.matcher.write().dl("o74").await.deduct_order(order);
    } else {
        add_order_to_matcher_and_process(order, global);
    }
    Ok(())
}
