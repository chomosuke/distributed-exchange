use lib::{interfaces::UserID, read_writer::ReadWriter, GResult};
use std::{net::SocketAddr, str::FromStr};
use structopt::StructOpt;
use tokio::{net::TcpStream, time::{sleep, Duration}};

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
    users[0]
        .write_line(r#"{ "type": "U balance", "value": 10000 }"#)
        .await?;
    assert_eq!(users[0].read_line().await?, r#""ok""#);
    users[0]
        .write_line(r#"{ "type": "C order", "value": { "order_type": "buy", "ticker": "Intel", "price": 15, "quantity": 50 } }"#)
        .await?;
    assert_eq!(users[0].read_line().await?, r#""ok""#);
    users[1]
        .write_line(r#"{ "type": "C stock", "value": { "ticker_id": "Intel", "quantity": 1000 } }"#)
        .await?;
    assert_eq!(users[1].read_line().await?, r#""ok""#);
    users[1]
        .write_line(r#"{ "type": "C order", "value": { "order_type": "sell", "ticker": "Intel", "price": 12, "quantity": 100 } }"#)
        .await?;
    assert_eq!(users[1].read_line().await?, r#""ok""#);

    sleep(Duration::from_millis(50)).await;

    // // tests
    // println!("User0:");
    // users[0].write_line(r#"{ "type": "R balance" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R stock" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R order" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R market" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // println!("User1:");
    // users[1].write_line(r#"{ "type": "R balance" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    // users[1].write_line(r#"{ "type": "R stock" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    // users[1].write_line(r#"{ "type": "R order" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    // users[1].write_line(r#"{ "type": "R market" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    //
    // // basic same node trade
    // users[0]
    //     .write_line(r#"{ "type": "C order", "value": { "order_type": "buy", "ticker": "AMD", "price": 15, "quantity": 50 } }"#)
    //     .await?;
    // assert_eq!(users[0].read_line().await?, r#""ok""#);
    // users[2]
    //     .write_line(r#"{ "type": "C stock", "value": { "ticker_id": "AMD", "quantity": 1000 } }"#)
    //     .await?;
    // assert_eq!(users[2].read_line().await?, r#""ok""#);
    // users[2]
    //     .write_line(r#"{ "type": "C order", "value": { "order_type": "sell", "ticker": "AMD", "price": 12, "quantity": 100 } }"#)
    //     .await?;
    // assert_eq!(users[2].read_line().await?, r#""ok""#);
    //
    // sleep(Duration::from_millis(50)).await;
    //
    // // tests
    // println!("User0:");
    // users[0].write_line(r#"{ "type": "R balance" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R stock" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R order" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R market" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    //
    // println!("User2:");
    // users[2].write_line(r#"{ "type": "R balance" }"#).await?;
    // println!("{}", users[2].read_line().await?);
    // users[2].write_line(r#"{ "type": "R stock" }"#).await?;
    // println!("{}", users[2].read_line().await?);
    // users[2].write_line(r#"{ "type": "R order" }"#).await?;
    // println!("{}", users[2].read_line().await?);
    // users[2].write_line(r#"{ "type": "R market" }"#).await?;
    // println!("{}", users[2].read_line().await?);
    //
    // // three way two node trade
    // users[0]
    //     .write_line(r#"{ "type": "C order", "value": { "order_type": "buy", "ticker": "AMD", "price": 15, "quantity": 20 } }"#)
    //     .await?;
    // assert_eq!(users[0].read_line().await?, r#""ok""#);
    // users[1]
    //     .write_line(r#"{ "type": "U balance", "value": 10000 }"#)
    //     .await?;
    // assert_eq!(users[1].read_line().await?, r#""ok""#);
    // users[1]
    //     .write_line(r#"{ "type": "C order", "value": { "order_type": "buy", "ticker": "AMD", "price": 12, "quantity": 100 } }"#)
    //     .await?;
    // assert_eq!(users[1].read_line().await?, r#""ok""#);
    //
    // sleep(Duration::from_millis(50)).await;
    //
    // // tests
    // println!("User0:");
    // users[0].write_line(r#"{ "type": "R balance" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R stock" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R order" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // users[0].write_line(r#"{ "type": "R market" }"#).await?;
    // println!("{}", users[0].read_line().await?);
    // println!("User1:");
    // users[1].write_line(r#"{ "type": "R balance" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    // users[1].write_line(r#"{ "type": "R stock" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    // users[1].write_line(r#"{ "type": "R order" }"#).await?;
    // println!("{}", users[1].read_line().await?);
    // users[1].write_line(r#"{ "type": "R market" }"#).await?;
    // println!("{}", users[1].read_line().await?);

    // say bye
    for mut user in users {
        user.write_line("\"bye\"").await?;
    }

    Ok(())
}
