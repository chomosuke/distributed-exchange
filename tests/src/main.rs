use lib::{read_writer::ReadWriter, GResult, interfaces::UserID};
use std::{net::SocketAddr, str::FromStr};
use structopt::StructOpt;
use tokio::net::TcpStream;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,
}

#[tokio::main]
async fn main() -> GResult<()> {
    let Args { coordinator } = Args::from_args();

    let mut user_ids = Vec::<UserID>::new();

    for i in 0..3 {
        let mut rw = ReadWriter::new(TcpStream::connect(coordinator).await?);
        rw.write_line("\"C account\"").await?;
        user_ids.push(serde_json::from_str(&rw.read_line().await?)?);
        println!("user{i} created: {}", user_ids[i]);
    }

    let mut users = Vec::<ReadWriter>::new();
    for user_id in user_ids {
        let mut rw = ReadWriter::new(TcpStream::connect(coordinator).await?);
        rw.write_line(&serde_json::to_string(&user_id)?).await?;
        let addr: SocketAddr = SocketAddr::from_str(&rw.read_line().await?)?;
        println!("Obtained node addr: {addr} for user: {user_id}");
        let mut rw = ReadWriter::new(TcpStream::connect(addr).await?);
        rw.write_line(&serde_json::to_string(&user_id)?).await?;
        users.push(rw);
        println!("Connection established");
    }

    // basic two node trade

    for mut user in users {
        user.write_line("\"bye\"").await?;
    }

    Ok(())
}
