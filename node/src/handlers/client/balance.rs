use std::{error::Error, sync::Arc};

use read_writer::ReadWriter;

use crate::Global;

use super::{Req, UserID};

pub async fn handler(
    user_id: UserID,
    req: Req,
    mut rw: ReadWriter,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    todo!()
}
