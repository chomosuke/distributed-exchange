use crate::{
    order::{process_order, OrderOrigin, OrderUpdate},
    Global,
};
use lib::{lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::sync::Arc;

pub async fn handler(req: Value, _rw: &mut ReadWriter, global: &Arc<Global>) -> GResult<()> {
    let OrderUpdate { deduct, order } = serde_json::from_value(req)?;
    let global = Arc::clone(global);
    tokio::spawn(async move {
        if deduct {
            println!("deduct order recieved {:?}", order.clone());
            global.matcher.write().dl("o74").await.deduct_order(order);
        } else {
            process_order(order, OrderOrigin::Local, &global)
                .await
                .expect("Process order failed");
        }
    });
    Ok(())
}
