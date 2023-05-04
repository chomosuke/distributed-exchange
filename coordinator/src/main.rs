use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use std::io::{BufWriter, Write};
use std::net::{SocketAddr, TcpListener};

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    port: u16,
}

#[derive(Serialize, Deserialize)]
struct ServerRecord {
    id: i32,
    address: SocketAddr,
}

fn main() {
    let args = Args::from_args();

    let ip_port = format!("127.0.0.1:{}", args.port);
    println!("Starting coordinator on {}", ip_port);
    let receiver_listener: TcpListener = TcpListener::bind(ip_port).expect("Failed to bind");

    let mut address_book: Vec<ServerRecord> = Vec::new();
    let mut next_id: i32 = 0;

    // Infinite loop to accept connections
    for stream in receiver_listener.incoming() {
        match stream {
            Err(e) => {
                eprintln!("Error receiving connection from a new server: {}", e);
            }

            Ok(stream) => {
                let s: ServerRecord = ServerRecord {
                    id: next_id,
                    address: stream
                        .peer_addr()
                        .expect("Couldn't connect to the node..."),
                };
                next_id += 1;

                // Write a list of the other clients to the conneciton Id
                let mut writer = BufWriter::new(stream);

                let buffer_string: String = format!("{}\n", &address_book.len());
                writer
                    .write(format!("{}\n", buffer_string).as_bytes())
                    .expect("Error writing to buffer");

                for record in &address_book {
                    let json_string: String = serde_json::to_string(&record)
                        .expect("Serialisation failed");
                    let buffer_string: String = format!("{}|", json_string);
                    writer
                        .write(buffer_string.as_bytes())
                        .expect("Error writing to buffer");
                }

                println!("{:?}", writer);

                // stream.write(&buffer[..bytes_written])
                writer.flush().expect("Error flushing buffer");

                // Create a new nodeid, store (NodeId, addr:port) in address_book
                address_book.push(s);
            }
        }
    }
}
