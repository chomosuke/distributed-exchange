use super::{Req, UserID};
use crate::Global;
use lib::read_writer::ReadWriter;
use std::{error::Error, sync::Arc};

pub async fn handler(
    user_id: &UserID,
    req: &Req,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    todo!()
}

