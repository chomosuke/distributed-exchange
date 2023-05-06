mod scanner;

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
    interfaces::{CentCount, Quantity, Ticker, UserID},
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

    loop {
        print_actions();
        match handle_command_logged_out(&mut scanner, &ip_port).await {
            ApplicationFlow::Break => break,
            ApplicationFlow::Continue => (),
            ApplicationFlow::LoginToNode(new_node_socket, account_id) => {
                // logged in
                let mut node_rw: ReadWriter = connect_to_server(&new_node_socket).await;

                loop {
                    print_account_actions(&account_id);
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
    println!("\nChoose an action:");
    println!("  c                Create a new account");
    println!("  l <account_id>   Login with your Account ID");
    println!("  q                Exit the application");
}

// actions when logged in to a user account
fn print_account_actions(account_id: &String) {
    println!("\nHello, User {}", account_id);
    println!("\nChoose an action:");
    println!("  b <ticker> <price> <quantity>  Submit a buy order");
    println!("  s <ticker> <price> <quantity>  Submit a sell order");
    println!("  c <ticker> <price> <quantity>  Cancel an order");
    println!("  o                              View your submitted orders");
    println!("  a                              View current account details");
    println!("  p                              View current stock prices");
    println!("  d <amount>                     Deposit cash");
    println!("  w <amount>                     Withdraw cash");
    // println!("  i <ticker> <quantity>          IPO new stock");
    println!("  !                              Delete your account permanently");
    println!("  q                              Exit the application");
}

async fn connect_to_server(ip_port: &SocketAddr) -> ReadWriter {
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
            scanner.clear();
        }
    }
    scanner.clear();
    ApplicationFlow::Continue
}

async fn handle_command_logged_in(scanner: &mut Scanner, rw: &mut ReadWriter) -> ApplicationFlow {
    match scanner.next::<String>().as_str() {
        "b" => {
            //Submit a buy order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationFlow::Continue;
            }
            let ticker = scanner.next::<Ticker>();
            if scanner.is_empty() {
                eprintln!("Invalid input after ticker: Expected <price> <quantity>");
                return ApplicationFlow::Continue;
            }
            let price = scanner.next::<CentCount>();
            if scanner.is_empty() {
                eprintln!("Invalid input after price: Expected <quantity>");
                return ApplicationFlow::Continue;
            }
            let quantity = scanner.next::<Quantity>();
            if !scanner.is_empty() {
                eprintln!("Unexpected input after quantity: ");
                print_remaining_input(scanner);
                scanner.clear();
                return ApplicationFlow::Continue;
            }
            submit_buy(&ticker, &price, &quantity).await;
        }
        "s" => {
            //Submit a sell order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationFlow::Continue;
            }
            let ticker = scanner.next::<Ticker>();
            if scanner.is_empty() {
                eprintln!("Invalid input after ticker: Expected <price> <quantity>");
                return ApplicationFlow::Continue;
            }
            let price = scanner.next::<CentCount>();
            if scanner.is_empty() {
                eprintln!("Invalid input after price: Expected <quantity>");
                return ApplicationFlow::Continue;
            }
            let quantity = scanner.next::<Quantity>();
            if !scanner.is_empty() {
                eprintln!("Unexpected input after quantity: ");
                print_remaining_input(scanner);
                scanner.clear();
                return ApplicationFlow::Continue;
            }
            submit_sell(&ticker, &price, &quantity).await;
        }
        "c" => {
            //Cancel an order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
            }
        }
        "o" => { //See your submitted orders
        }
        "a" => { //See current account details
        }
        "p" => { //See current stock prices
        }
        "q" => {
            // Exit the application
            scanner.clear();
            rw.write_line(r#""bye""#).await.expect("Failed to send goodbye to node");
            println!("Shutting down.");
            return ApplicationFlow::Break;
        }
        _other => {
            eprintln!("Unknown command: {}", _other);
            scanner.clear();
        }
    }
    scanner.clear();
    ApplicationFlow::Continue
}

async fn create_account(ip_port: &SocketAddr) -> GResult<UserID> {
    let mut rw: ReadWriter = connect_to_server(ip_port).await;
    rw.write_line(r#""C Account""#).await?;
    let userid: UserID = serde_json::from_str(&rw.read_line().await?)?;
    Ok(userid)
}

async fn login(ip_port: &SocketAddr, account_id: &str) -> GResult<SocketAddr> {
    // let sp: Vec<&str> = account_id.split('.').collect();
    // if sp.len() != 2 {
    //     return Err(Box::from("Incorrect format for Account ID"));
    // }

    // let (id, node) = (sp[0], sp[1]);
    let user_id: UserID =
        UserID::from_str(account_id).map_err(|_| "Invalid format for User ID")?;

    println!("user_id: {}", user_id);

    let message: String =
        serde_json::to_string::<UserID>(&user_id).map_err(|_| "Error serialising User ID")?;

    println!("message: {}", message);

    let mut rw: ReadWriter = connect_to_server(ip_port).await;
    rw.write_line(&message).await?;

    let node_address = SocketAddr::from_str(&rw.read_line().await?)?;

    println!("node_address: {}", node_address);

    Ok(node_address)
}

async fn submit_buy(ticker: &Ticker, price: &CentCount, quantity: &Quantity) {
    todo!()
    // submit_trade()
}

async fn submit_sell(ticker: &Ticker, price: &CentCount, quantity: &Quantity) {
    todo!()
}

async fn submit_trade() {
    //TODO: t: Trade) {
    todo!()
}

fn cancel_trade() {
    todo!()
}

fn print_balance() {
    todo!()
}

fn print_orders() {
    todo!()
}

fn deposit_cash() {
    todo!()
}

fn withdraw_cash() {
    todo!()
}

fn delete_account() {
    todo!()
}
