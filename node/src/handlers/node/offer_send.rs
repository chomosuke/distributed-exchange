use super::PendingOffer;
use crate::{matcher::Trade, Global};
use lib::{read_writer::ReadWriter, GResult};
use std::sync::Arc;

pub async fn handler(
    trade: Trade,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
    pending_offer: &mut PendingOffer,
) -> GResult<String> {
    todo!()
}
