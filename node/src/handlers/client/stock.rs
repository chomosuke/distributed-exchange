use super::{Req, UserID, CRUD};
use crate::Global;
use lib::{read_writer::ReadWriter, GResult};
use std::{error::Error, sync::Arc};

pub async fn handler(
    user_id: &UserID,
    Req { crud, value, .. }: &Req,
    rw: &mut ReadWriter,
    global: &Arc<Global>,
) -> GResult<String> {
    todo!()
}
