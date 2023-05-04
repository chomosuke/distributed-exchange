use std::error::Error;

use tokio::{
    io::{AsyncBufReadExt, BufReader, BufWriter, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};

pub struct ReadWriter<'a> {
    reader: BufReader<ReadHalf<'a>>,
    writer: BufWriter<WriteHalf<'a>>,
}

impl<'a> ReadWriter<'a> {
    pub fn new(socket: &'a mut TcpStream) -> Self {
        let (read_half, write_half) = socket.split();
        Self {
            reader: BufReader::new(read_half),
            writer: BufWriter::new(write_half),
        }
    }

    pub async fn write_line(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.writer.write_all(s.as_bytes()).await?;
        self.writer.write_u8(b'\n').await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn read_line(&mut self) -> Result<String, Box<dyn Error>> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        Ok(line)
    }
}
