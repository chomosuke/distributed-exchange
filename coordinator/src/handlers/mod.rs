use std::{error::Error, str::FromStr, sync::Arc};
use read_writer::ReadWriter;
use tokio::net::TcpStream;

use crate::Global;

mod client;
mod node;

pub use node::Message;

pub async fn handler(mut socket: TcpStream, global: Arc<Global>) -> Result<String, Box<dyn Error>> {
    let mut rw = ReadWriter::new(&mut socket);
    let first_line = rw.read_line().await?;

    if let Ok(first_line) = client::FirstLine::from_str(&first_line) {
        client::handler(first_line, rw, global).await
    } else if let Ok(first_line) = node::FirstLine::from_str(&first_line) {
        node::handler(first_line, rw, global).await
    } else {
        Err(format!("{first_line} is not a valid request").into())
    }
}
