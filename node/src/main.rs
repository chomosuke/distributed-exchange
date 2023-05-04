use serde::{Deserialize, Serialize};
use std::env::args;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

fn main() {
    contact_coordinator();
}

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
