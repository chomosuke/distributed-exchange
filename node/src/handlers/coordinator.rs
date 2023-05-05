use std::{error::Error, net::SocketAddr, sync::Arc};

use lib::read_writer::ReadWriter;
use serde::Deserialize;
use tokio::net::TcpStream;

use crate::{Global, NodeID, Node};

use super::{get_type, node, client::UserID};

#[derive(Deserialize)]
struct JoinedReq {
    id: NodeID,
    addr: SocketAddr,
}

pub async fn handler(mut rw: ReadWriter, global: Arc<Global>) -> Result<String, Box<dyn Error>> {
    let req = rw.read_line().await?;
    let t = get_type(&req)?;
    let this_id = (*global.state.read().await).get_id();

    match t.as_str() {
        "joined" => {
            let JoinedReq { id: other_id, addr } = serde_json::from_str(&req)?;

            // modify others
            global
                .others
                .write()
                .await
                .insert(other_id, Node::DisConnected(addr));

            let mut rw = ReadWriter::new(
                TcpStream::connect(addr)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to connect to node {}", req)),
            );

            rw.write_line(&this_id.to_string()).await?;
            let global = Arc::clone(&global);
            tokio::spawn(async move {
                match node::handler(node::FirstLine(other_id), rw, global).await {
                    Ok(msg) => println!("Connection terminated successfully: {msg}"),
                    Err(e) => eprintln!("Error: {e}"),
                }
            });
        }
        "C account" => {
            let acc_id = global.state.write().await.create_account().await?;
            rw.write_line(&serde_json::to_string(&UserID { id: acc_id, node_id: this_id })?).await?;
        }
        t => return Err(Box::from(format!("Wrong type {}.", t,))),
    }
    Ok(format!("Handled request: {req}"))
}
