use std::io::{BufRead, BufReader};
use std::net::{TcpStream, SocketAddr};
use structopt::StructOpt;


#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,

    #[structopt(short, long)]
    port: u16,
}

fn main() {
    let args = Args::from_args();

    println!("Contacting coordinator on {}", args.coordinator);

    // Connect to coordinator
    let stream = TcpStream::connect(args.coordinator).expect("Failed to connect");
    let mut reader = BufReader::new(&stream);
    let mut buffer: Vec<u8> = Vec::new();

    // Get Id from coordinator
    reader
        .read_until(b'\n', &mut buffer)
        .ok()
        .expect("Error reading");
    let buffer_string: String = String::from_utf8(buffer)
        .ok()
        .expect("Error unwrapping string");
    let buffer_string: String = format!("{}", buffer_string.trim_end());

    let server_id: i32 = buffer_string
        .parse::<i32>()
        .ok()
        .expect("Error parsing server id number");

    println!("Server Id: {}", server_id);
}
