use std::{error::Error, sync::Arc};

use read_writer::ReadWriter;

use crate::Global;

use super::Req;

pub async fn handler(
    req: Req,
    mut rw: ReadWriter<'_>,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
}

