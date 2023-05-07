use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::GResult;

pub struct ReadWriter {
    peer_addr: Result<SocketAddr, String>,
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl ReadWriter {
    pub fn new(socket: TcpStream) -> Self {
        let peer_addr = socket.peer_addr().map_err(|e| e.to_string());
        let (r, w) = socket.into_split();
        Self {
            peer_addr,
            reader: BufReader::new(r),
            writer: BufWriter::new(w),
        }
    }

    pub async fn write_line(&mut self, s: &str) -> GResult<()> {
        println!("write_line: {s}");
        self.writer.write_all(s.as_bytes()).await?;
        self.writer.write_u8(b'\n').await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn read_line(&mut self) -> GResult<String> {
        let mut line = Vec::new();
        self.reader.read_until(b'\n', &mut line).await?;
        line.pop();
        Ok(String::from_utf8(line)?)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr, String> {
        self.peer_addr.clone()
    }
}
