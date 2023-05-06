#![allow(clippy::new_without_default)]
mod handlers;
mod state;

use crate::{handlers::handler, state::State};
use lib::read_writer::ReadWriter;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::net::TcpListener;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    port: u16,

    #[structopt(short = "d", long)]
    persistent_dir: String,
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let ip_port = format!("127.0.0.1:{}", args.port);
    println!("Starting coordinator on {ip_port}");
    let listener: TcpListener = TcpListener::bind(ip_port).await.expect("Failed to bind");

    let global: Arc<State> = Arc::new(State::new_or_restore(args.persistent_dir).await);

    loop {
        let rw = match listener.accept().await {
            Ok((socket, _)) => ReadWriter::new(socket),
            Err(e) => {
                eprintln!("Error receiving connection from a new connection: {e}");
                continue;
            }
        };
        let global = Arc::clone(&global);
        tokio::spawn(async {
            match handler(rw, global).await {
                Ok(msg) => println!("Connection terminated successfully: {msg}"),
                Err(e) => eprintln!("Error with connection: {e}"),
            }
        });
    }
}
