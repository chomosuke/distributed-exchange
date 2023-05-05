use std::error::Error;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub struct ReadWriter {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl ReadWriter {
    pub fn new(socket: TcpStream) -> Self {
        let (r, w) = socket.into_split();
        Self {
            reader: BufReader::new(r),
            writer: BufWriter::new(w),
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
