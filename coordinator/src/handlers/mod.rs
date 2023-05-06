use lib::{read_writer::ReadWriter, GResult};
use std::{str::FromStr, sync::Arc};

use crate::State;

pub mod client;
pub mod node;

pub async fn handler(
    mut rw: ReadWriter,
    state: Arc<State>,
) -> GResult<String> {
    let first_line = rw.read_line().await?;

    if let Ok(first_line) = client::FirstLine::from_str(&first_line) {
        client::handler(first_line, rw, state).await
    } else if let Ok(first_line) = node::FirstLine::from_str(&first_line) {
        node::handler(first_line, rw, state).await
    } else {
        rw.write_line("bad request").await?;
        Err(format!("{first_line} is not a valid request").into())
    }
}
