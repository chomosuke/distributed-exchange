use super::{get_value_type, node};
use crate::{Global, Node, NodeID};
use lib::{interfaces::UserID, lock::DeadLockDetect, read_writer::ReadWriter, GResult};
use serde::Deserialize;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpStream;

#[derive(Deserialize)]
struct JoinedReq {
    id: NodeID,
    addr: SocketAddr,
}

pub async fn handler(mut rw: ReadWriter, global: Arc<Global>) -> GResult<String> {
    let this_id = (*global.state.read().dl("co17").await).get_id();

    loop {
        let req = rw.read_line().await?;
        let (req_type, _) = get_value_type(&req)?;

        match req_type.as_str() {
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
                        Ok(msg) => println!("Connection terminated with node: {msg}"),
                        Err(e) => eprintln!("Error: {e}"),
                    }
                });
            }
            "C account" => {
                let acc_id = global
                    .state
                    .write()
                    .dl("co47")
                    .await
                    .create_account()
                    .await?;
                rw.write_line(&serde_json::to_string(&UserID {
                    id: acc_id,
                    node_id: this_id,
                })?)
                .await?;
            }
            req_type => return Err(Box::from(format!("Wrong type {}.", req_type))),
        }
        println!("Handled request from coordinator: {req}")
    }
}
