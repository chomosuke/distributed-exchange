use std::{error::Error, net::SocketAddr};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

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

    pub fn peer_addr(&self) -> Result<SocketAddr, String> {
        self.peer_addr.clone()
    }
}
