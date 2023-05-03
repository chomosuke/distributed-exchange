// use std::collections::HashMap;
use std::env::args;
use std::io::{Write, BufWriter, BufReader, BufRead};
use std::net::{SocketAddr, TcpListener, TcpStream};
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
struct ServerRecord {
    id: i32,
    address: SocketAddr
}

fn main() {
    let arguments: Vec<String> = args().collect();

    if arguments.len() < 2 {
        eprintln!("Expected flag '-c' (Coordinator mode) or '-n' (Non-coordinator mode)");
        return;
    }

    if arguments[1].eq("-c") {
        coordinate();
    } else if arguments[1].eq("-n") {
        contact_coordinator();
    } else {
        eprintln!("Expected flag '-c' (Coordinator mode) or '-n' (Non-coordinator mode) 2");
    }
}

fn coordinate() {
    println!("Starting first server at IP {}, Port {}", "127.0.0.1", "8000");
    let receiver_listener: TcpListener = TcpListener::bind("127.0.0.1:8000")
        .expect("Failed to bind");

    let mut address_book: Vec<ServerRecord> = Vec::new();
    let mut next_id: i32 = 0;

    // Infinite loop to accept connections
    for stream in receiver_listener.incoming() {
        match stream {
            Err(e) => {eprintln!("Error receiving connection from a new server: {}", e);}

            Ok(stream) => {
                let s: ServerRecord = ServerRecord {
                    id: next_id,
                    address: stream.peer_addr().ok()
                        .expect("Couldn't connect to the server...")
                };
                next_id += 1;


                // Write a list of the other clients to the conneciton Id
                let mut writer = BufWriter::new(stream);

                let buffer_string: String = format!("{}\n", &address_book.len());
                writer.write(format!("{}\n", buffer_string).as_bytes()).ok()
                    .expect("Error writing to buffer");


                for record in &address_book {
                    let json_string: String = serde_json::to_string(&record).ok()
                        .expect("Serialisation failed");
                    let buffer_string: String = format!("{}|", json_string);
                    writer.write(buffer_string.as_bytes()).ok()
                        .expect("Error writing to buffer");
                }

                println!("{:?}", writer);

                // stream.write(&buffer[..bytes_written])
                writer.flush()
                    .expect("Error flushing buffer");

                 // Create a new nodeid, store (NodeId, addr:port) in address_book
                address_book.push(s);
            }
        }
    }
}

    // Create Vec of other servers from messages recieved from coordinator
    // Establish connection with each other server
fn contact_coordinator() {
    println!("Connecting to coordinator server at IP {}, Port {}", "127.0.0.1", "8000");

    // Connect to coordinator
    let stream = TcpStream::connect("127.0.0.1:8000").ok()
        .expect("Failed to connect");
    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();

    // Get Id from coordinator
    reader.read_until(b'\n', &mut buffer).ok().expect("Error reading");
    let buffer_string: String = String::from_utf8(buffer).ok()
        .expect("Error unwrapping string");
    let buffer_string: String = format!("{}", buffer_string.trim_end());

    let server_id: i32 = buffer_string.parse::<i32>().ok()
        .expect("Error parsing server id number");

    println!("Server Id: {}", server_id);

}
