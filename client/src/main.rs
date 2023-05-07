mod scanner;

use serde_json::{de::Read, json, Map, Value};
use std::{net::SocketAddr, str::FromStr};
use structopt::StructOpt;

#[allow(unused_imports)]
use scanner::*;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,
}

use lib::{
    interfaces::{CentCount, OrderReq, OrderType, Quantity, Ticker, UserID},
    read_writer::ReadWriter,
    GResult,
};
use tokio::net::TcpStream;

enum ApplicationFlow {
    LoginToNode(SocketAddr, String),
    Continue,
    Break,
}

const HEADER_TEXT: &str = r"
==================================================================================================
 _______   __             __                __  __                    __                      __
/       \ /  |           /  |              /  |/  |                  /  |                    /  |
$$$$$$$  |$$/   _______ _$$ |_     ______  $$/ $$ |____   __    __  _$$ |_     ______    ____$$ |
$$ |  $$ |/  | /       / $$   |   /      \ /  |$$      \ /  |  /  |/ $$   |   /      \  /    $$ |
$$ |  $$ |$$ |/$$$$$$$/$$$$$$/   /$$$$$$  |$$ |$$$$$$$  |$$ |  $$ |$$$$$$/   /$$$$$$  |/$$$$$$$ |
$$ |  $$ |$$ |$$      \  $$ | __ $$ |  $$/ $$ |$$ |  $$ |$$ |  $$ |  $$ | __ $$    $$ |$$ |  $$ |
$$ |__$$ |$$ | $$$$$$  | $$ |/  |$$ |      $$ |$$ |__$$ |$$ \__$$ |  $$ |/  |$$$$$$$$/ $$ \__$$ |
$$    $$/ $$ |/     $$/  $$  $$/ $$ |      $$ |$$    $$/ $$    $$/   $$  $$/ $$       |$$    $$ |
$$$$$$$/  $$/ $$$$$$$/    $$$$/  $$/       $$/ $$$$$$$/   $$$$$$/     $$$$/   $$$$$$$/  $$$$$$$/
                 __       __                      __                    __
                /  \     /  |                    /  |                  /  |
                $$  \   /$$ |  ______    ______  $$ |   __   ______   _$$ |_
                $$$  \ /$$$ | /      \  /      \ $$ |  /  | /      \ / $$   |
                $$$$  /$$$$ | $$$$$$  |/$$$$$$  |$$ |_/$$/ /$$$$$$  |$$$$$$/
                $$ $$ $$/$$ | /    $$ |$$ |  $$/ $$   $$<  $$    $$ |  $$ | __
                $$ |$$$/ $$ |/$$$$$$$ |$$ |      $$$$$$  \ $$$$$$$$/   $$ |/  |
                $$ | $/  $$ |$$    $$ |$$ |      $$ | $$  |$$       |  $$  $$/
                $$/      $$/  $$$$$$$/ $$/       $$/   $$/  $$$$$$$/    $$$$/
==================================================================================================
                        STONKS ONLY GO UP - Warren Buffet, probably
==================================================================================================";

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let ip_port: SocketAddr = args.coordinator;
    println!("Contacting coordinator at {ip_port}");

    // Launch
    println!("{}\n", HEADER_TEXT);
    println!("Welcome to the Distributed Stock Exchange!\n");

    let mut scanner: Scanner = Scanner::new();

    print_actions();
    loop {
        match handle_command_logged_out(&mut scanner, &ip_port).await {
            ApplicationFlow::Break => break,
            ApplicationFlow::Continue => (),
            ApplicationFlow::LoginToNode(new_node_socket, account_id) => {
                // logged in
                let mut node_rw: ReadWriter = connect_to_server(&new_node_socket).await;

                send_user_id(&mut node_rw, account_id.as_str())
                    .await
                    .expect("Error sending User ID");

                println!("-- Logged in as User {} --", &account_id);
                print_account_actions();
                loop {
                    match handle_command_logged_in(&mut scanner, &mut node_rw).await {
                        ApplicationFlow::LoginToNode(_, _) => {
                            // If this happens, something very wrong is going on!
                            eprintln!("Error: Unexpected Login. Already logged in.");
                            break;
                        }
                        ApplicationFlow::Break => break,
                        ApplicationFlow::Continue => (),
                    }
                }
                break;
            }
        }
    }
}

// actions when not logged in
fn print_actions() {
    print!(
        r#"
Choose an action:
  c                Create a new account
  l <account_id>   Login with your Account ID
  q                Exit the application

"#
    );
}

// actions when logged in to a user account
fn print_account_actions() {
    print!(
        r#"

Choose an action:
  b <ticker> <price> <quantity>  Submit a buy order
  s <ticker> <price> <quantity>  Submit a sell order
  o                              View your submitted orders
  a                              View current cash account balance
  c <amount>                     Set cash account balance
  p                              View your current stock portfolio
  i <ticker> <quantity>          IPO: Add new stock to account
  q                              Exit the application

"#
    );
}

async fn connect_to_server(ip_port: &SocketAddr) -> ReadWriter {
    println!("Connecting to {ip_port}");
    ReadWriter::new(
        TcpStream::connect(ip_port)
            .await
            .expect("Failed to connect to server"),
    )
}

fn print_remaining_input(scanner: &mut Scanner) {
    while !scanner.is_empty() {
        eprint!("{} ", scanner.next::<String>());
    }
    eprintln!();
    scanner.clear();
}

async fn handle_command_logged_out(scanner: &mut Scanner, ip_port: &SocketAddr) -> ApplicationFlow {
    match scanner.next::<String>().as_str() {
        "c" => {
            //Create a new account
            if !scanner.is_empty() {
                eprintln!("Unexpected input after c: ");
                print_remaining_input(scanner);
            } else {
                let res = create_account(ip_port)
                    .await
                    .expect("Error creating account");
                println!("New account created: {}.{}", res.node_id, res.id);
            }
        }
        "l" => {
            //Login with your Account ID
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <account_id>");
            } else {
                let entered_account_id = scanner.next::<String>();

                if !scanner.is_empty() {
                    eprint!("Unexpected input after account_id: ");
                    print_remaining_input(scanner);
                } else {
                    match login(ip_port, &entered_account_id).await {
                        Err(e) => {
                            scanner.clear();
                            eprintln!("{}", e);
                            return ApplicationFlow::Continue;
                        }
                        Ok(socket_addr) => {
                            return ApplicationFlow::LoginToNode(socket_addr, entered_account_id);
                        }
                    }
                }
            }
        }
        "q" => {
            // Exit the application
            scanner.clear();
            println!("Shutting down.");
            return ApplicationFlow::Break;
        }
        _other => {
            eprintln!("Unknown command: {}", _other);
            print_actions();
            scanner.clear();
        }
    }
    scanner.clear();
    ApplicationFlow::Continue
}

fn get_tpq_input(scanner: &mut Scanner) -> GResult<(Ticker, CentCount, Quantity)> {
    if scanner.is_empty() {
        return Err(Box::from(
            "Invalid input: Expected <ticker> <price> <quantity>",
        ));
    }
    let ticker = scanner.next::<Ticker>();
    if scanner.is_empty() {
        return Err(Box::from(
            "Invalid input after ticker: Expected <price> <quantity>",
        ));
    }
    let price = scanner.next::<CentCount>();
    if scanner.is_empty() {
        return Err(Box::from("Invalid input after price: Expected <quantity>"));
    }
    let quantity = scanner.next::<Quantity>();
    if !scanner.is_empty() {
        print_remaining_input(scanner);
        return Err(Box::from("Unexpected input after quantity: "));
    }
    Ok((ticker, price, quantity))
}

fn get_tq_input(scanner: &mut Scanner) -> GResult<(Ticker, Quantity)> {
    if scanner.is_empty() {
        return Err(Box::from("Invalid input: Expected <ticker> <quantity>"));
    }
    let ticker = scanner.next::<Ticker>();
    if scanner.is_empty() {
        return Err(Box::from("Invalid input after ticker: Expected <quantity>"));
    }
    let quantity = scanner.next::<Quantity>();
    if !scanner.is_empty() {
        print_remaining_input(scanner);
        return Err(Box::from("Unexpected input after quantity: "));
    }
    Ok((ticker, quantity))
}

fn get_q_input(scanner: &mut Scanner) -> GResult<Quantity> {
    if scanner.is_empty() {
        return Err(Box::from("Invalid input: Expected <quantity>"));
    }
    let quantity = scanner.next::<Quantity>();
    if !scanner.is_empty() {
        print_remaining_input(scanner);
        return Err(Box::from("Unexpected input after quantity: "));
    }
    Ok(quantity)
}

async fn handle_command_logged_in(scanner: &mut Scanner, rw: &mut ReadWriter) -> ApplicationFlow {
    match scanner.next::<String>().as_str() {
        "b" => {
            //Submit a buy order
            match get_tpq_input(scanner) {
                Err(e) => {
                    eprintln!("{}", e);
                }
                Ok((ticker, price, quantity)) => {
                    match submit_order(rw, OrderType::Buy, ticker, price, quantity).await {
                        Ok(res) => {
                            if res == "ok" {
                                println!("Buy order submitted");
                            } else {
                                println!("{res}");
                            }
                        }
                        Err(e) => {
                            eprintln!("{e}");
                        }
                    }
                }
            }
        }
        "s" => {
            //Submit a sell order
            match get_tpq_input(scanner) {
                Err(e) => {
                    eprintln!("{}", e);
                }
                Ok((ticker, price, quantity)) => {
                    match submit_order(rw, OrderType::Sell, ticker, price, quantity).await {
                        Ok(res) => {
                            if res == "ok" {
                                println!("Sell order submitted");
                            } else {
                                println!("{res}");
                            }
                        }
                        Err(e) => {
                            eprintln!("{e}");
                        }
                    }
                }
            }
        }
        "o" => {
            //See your submitted orders
            if !scanner.is_empty() {
                eprint!("Unexpected input: ");
                print_remaining_input(scanner);
            }
            print_orders(rw).await.expect("Error printing orders");
        }
        "a" => {
            //See current account details
            if !scanner.is_empty() {
                eprint!("Unexpected input: ");
                print_remaining_input(scanner);
            }
            print_balance(rw)
                .await
                .expect("Error printing account balance");
        }
        "c" => {
            //Set cash account balance
            match get_q_input(scanner) {
                Err(e) => {
                    eprintln!("{}", e);
                }
                Ok(quantity) => match set_balance(rw, quantity).await {
                    Ok(res) => {
                        if res == "ok" {
                            println!("Account balance updated");
                        } else {
                            println!("{res}");
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                },
            }
        }
        "p" => {
            //See current stock portfolio
            if !scanner.is_empty() {
                eprint!("Unexpected input: ");
                print_remaining_input(scanner);
            }
            print_portfolio(rw).await.expect("Error printing portfolio");
        }
        "i" => {
            //IPO
            match get_tq_input(scanner) {
                Err(e) => {
                    eprintln!("{}", e);
                }
                Ok((ticker, quantity)) => match ipo(rw, ticker, quantity).await {
                    Ok(res) => {
                        if res == "ok" {
                            println!("IPO submitted");
                        } else {
                            println!("{res}");
                        }
                    }
                    Err(e) => {
                        eprintln!("{e}");
                    }
                },
            }
        }
        "q" => {
            // Exit the application
            scanner.clear();
            rw.write_line(r#""bye""#)
                .await
                .expect("Failed to send goodbye to node");
            println!("Shutting down.");
            return ApplicationFlow::Break;
        }
        _other => {
            eprintln!("Unknown command: {}", _other);
            print_account_actions();
            scanner.clear();
        }
    }
    scanner.clear();
    ApplicationFlow::Continue
}

async fn create_account(ip_port: &SocketAddr) -> GResult<UserID> {
    let mut rw: ReadWriter = connect_to_server(ip_port).await;
    rw.write_line(r#""C account""#).await?;
    let userid: UserID = serde_json::from_str(&rw.read_line().await?)?;
    Ok(userid)
}

async fn send_user_id(rw: &mut ReadWriter, account_id: &str) -> GResult<()> {
    let user_id: UserID = UserID::from_str(account_id).map_err(|_| "Invalid format for User ID")?;
    let message: String =
        serde_json::to_string::<UserID>(&user_id).map_err(|_| "Error serialising User ID")?;
    rw.write_line(&message)
        .await
        .map_err(|_| "Error writing User ID to ReadWriter")?;
    Ok(())
}

async fn login(ip_port: &SocketAddr, account_id: &str) -> GResult<SocketAddr> {
    let mut rw: ReadWriter = connect_to_server(ip_port).await;

    send_user_id(&mut rw, account_id).await?;

    let node_address = SocketAddr::from_str(&rw.read_line().await?)?;
    Ok(node_address)
}

async fn submit_order(
    rw: &mut ReadWriter,
    order_type: OrderType,
    ticker: Ticker,
    price: CentCount,
    quantity: Quantity,
) -> GResult<String> {
    let order_req: OrderReq = OrderReq {
        order_type,
        ticker,
        price,
        quantity,
    };
    let msg_json = json!({
        "type": "C order",
        "value": order_req
    });
    let message = serde_json::to_string(&msg_json).expect("Failed to build trade request");

    rw.write_line(&message).await?;

    let res: String = serde_json::from_str(&rw.read_line().await?)?;
    Ok(res)
}

async fn print_balance(rw: &mut ReadWriter) -> GResult<()> {
    let msg_json = json!({
        "type": "R balance"
    });
    let message = serde_json::to_string(&msg_json).expect("Failed to build balance request");

    rw.write_line(&message).await?;

    let res: u64 = serde_json::from_str(&rw.read_line().await?)?;
    println!("Current account balance: ${}.{}", res / 100, res % 100);
    Ok(())
}

async fn set_balance(rw: &mut ReadWriter, quantity: u64) -> GResult<String> {
    let msg_json = json!({
        "type": "U balance",
        "value": quantity
    });

    let message = serde_json::to_string(&msg_json).expect("Failed to build balance update request");

    rw.write_line(&message).await?;

    let res: String = serde_json::from_str(&rw.read_line().await?)?;
    Ok(res)
}

async fn print_portfolio(rw: &mut ReadWriter) -> GResult<()> {
    let msg_json = json!({
        "type": "R stock"
    });
    let message = serde_json::to_string(&msg_json).expect("Failed to build portfolio request");

    rw.write_line(&message).await?;

    let res: Map<String, Value> = serde_json::from_str(&rw.read_line().await?)?;

    println!("Current portfolio:");
    for (k, v) in res.iter() {
        println!(" {k}: {v}");
    }

    Ok(())
}

async fn print_orders(rw: &mut ReadWriter) -> GResult<()> {
    let msg_json = json!({
        "type": "R order"
    });
    let message = serde_json::to_string(&msg_json).expect("Failed to build order view request");

    rw.write_line(&message).await?;

    let res: Map<String, Value> = serde_json::from_str(&rw.read_line().await?)?;

    println!("Pending orders:");
    for (k, v) in res.iter() {
        // let buysell: Map<String, Value> = serde_json::from_value(v.clone())?;
        println!(" {k}: {v}");
    }

    Ok(())
}

async fn ipo(rw: &mut ReadWriter, ticker: Ticker, quantity: Quantity) -> GResult<String> {
    let msg_json = json!({
        "type": "C stock",
        "value": {
            "ticker_id": ticker,
            "quantity": quantity
        }
    });
    let message = serde_json::to_string(&msg_json).expect("Failed to build IPO request");

    rw.write_line(&message).await?;

    let res: String = serde_json::from_str(&rw.read_line().await?)?;
    Ok(res)
}
