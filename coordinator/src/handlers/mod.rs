use std::{error::Error, str::FromStr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader, BufWriter},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};

use crate::Global;

mod client;
mod node;

pub use node::Message;

struct ReadWriter<'a> {
    reader: BufReader<ReadHalf<'a>>,
    writer: BufWriter<WriteHalf<'a>>,
}

pub async fn handler(mut socket: TcpStream, global: Arc<Global>) -> Result<String, Box<dyn Error>> {
    let mut first_line = String::new();
    let (read_half, write_half) = socket.split();
    let mut rw = ReadWriter {
        reader: BufReader::new(read_half),
        writer: BufWriter::new(write_half),
    };
    rw.reader.read_line(&mut first_line).await?;

    if let Ok(first_line) = client::FirstLine::from_str(&first_line) {
        client::handler(first_line, rw, global).await
    } else if let Ok(first_line) = node::FirstLine::from_str(&first_line) {
        node::handler(first_line, rw, global).await
    } else {
        Err(format!("{first_line} is not a valid request").into())
    }
}
