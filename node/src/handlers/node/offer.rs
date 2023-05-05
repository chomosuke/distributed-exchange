use std::{error::Error, sync::Arc};

use lib::read_writer::ReadWriter;
use serde_json::Value;

use crate::Global;

use super::PendingOffer;

pub async fn handler(
    req: &Value,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
    pending_offer: &mut PendingOffer,
) -> Result<String, Box<dyn Error>> {
    todo!()
}
