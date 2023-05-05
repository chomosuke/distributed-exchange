use std::{error::Error, net::SocketAddr, sync::Arc};

use read_writer::ReadWriter;
use serde::Deserialize;
use tokio::net::TcpStream;

use crate::{Global, NodeID, Node};

use super::{get_type, node};

#[derive(Deserialize)]
struct JoinedReq {
    id: NodeID,
    addr: SocketAddr,
}

pub async fn handler(mut rw: ReadWriter, global: Arc<Global>) -> Result<String, Box<dyn Error>> {
    let req = rw.read_line().await?;
    let t = get_type(&req)?;

    match t.as_str() {
        "joined" => {
            let JoinedReq { id, addr } = serde_json::from_str(&req)?;

            // modify others
            global
                .others
                .write()
                .await
                .insert(id, Node::DisConnected(addr));

            let mut rw = ReadWriter::new(
                TcpStream::connect(addr)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to connect to node {}", req)),
            );

            rw.write_line(&id.to_string()).await?;
            let global = Arc::clone(&global);
            tokio::spawn(async move {
                match node::handler(node::FirstLine(id), rw, global).await {
                    Ok(msg) => println!("Connection terminated successfully: {msg}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
            });
        }
        "C account" => {
            todo!()
        }
        t => return Err(Box::from(format!("Wrong type {}.", t,))),
    }
    Ok(format!("Handled request: {req}"))
}
