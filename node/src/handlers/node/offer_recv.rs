use crate::Global;
use lib::{read_writer::ReadWriter, GResult};
use serde_json::Value;
use std::{error::Error, sync::Arc};

pub async fn handler(
    req: &Value,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
) -> GResult<String> {
    todo!()
}
