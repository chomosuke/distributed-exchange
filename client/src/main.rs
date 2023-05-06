mod scanner;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use structopt::StructOpt;

#[allow(unused_imports)]
use scanner::*;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,
}

use lib::interfaces::{CentCount, Quantity, Ticker, UserID};

enum ApplicationStatus {
    Continue,
    Complete,
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

fn main() {
    let args = Args::from_args();

    let ip_port = args.coordinator;
    println!("Contacting coordinator at {ip_port}");

    // Connect to coordinator
    // let mut socket = TcpStream::connect(ip_port).await.expect("Failed to bind");
    // ::bind(ip_port).await.expect("Failed to bind");
    // TcpStream

    // Launch
    println!("{}\n", HEADER_TEXT);
    println!("Welcome to the Distributed Stock Exchange!\n\n");

    let mut scanner: Scanner = Scanner::new();
    let mut account_id: String = String::new();

    loop {
        println!("Choose an action:");
        if account_id.is_empty() {
            // not logged in
            println!("  c                Create a new account");
            println!("  l <account_id>   Login with your Account ID");
            println!("  q                Exit the application");

            match handle_command_logged_out(&mut scanner, &mut account_id) {
                ApplicationStatus::Complete => break,
                ApplicationStatus::Continue => {}
            }
        } else {
            // logged in
            println!("Choose an action:");
            println!("  b <ticker> <price> <quantity>  Submit a buy order");
            println!("  s <ticker> <price> <quantity>  Submit a sell order");
            println!("  c <ticker> <price> <quantity>  Cancel an order");
            println!("  o                              View your submitted orders");
            println!("  a                              View current account details");
            println!("  p                              View current stock prices");
            println!("  d <amount>                     Deposit cash");
            println!("  w <amount>                     Withdraw cash");
            println!("  !                              Delete your account permanently");
            println!("  q                              Quit the application");

            // Going IPO

            match handle_command_logged_in(&mut scanner, &account_id) {
                ApplicationStatus::Complete => break,
                ApplicationStatus::Continue => {}
            }
        }
    }
}

fn create_account() {
    todo!();
}

fn login(account_id: &mut String, new_account_id: String) {
    todo!()
}

fn submit_buy(account_id: &String, ticker: &Ticker, price: &CentCount, quantity: &Quantity) {
    todo!()
}

fn submit_sell(account_id: &String, ticker: &Ticker, price: &CentCount, quantity: &Quantity) {
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

fn print_remaining_input(scanner: &mut Scanner) {
    while !scanner.is_empty() {
        eprint!("{} ", scanner.next::<String>());
    }
    eprintln!();
}

fn handle_command_logged_out(scanner: &mut Scanner, account_id: &mut String) -> ApplicationStatus {
    match scanner.next::<String>().as_str() {
        "c" => {
            //Create a new account
            if !scanner.is_empty() {
                eprintln!("Unexpected input after c: ");
                print_remaining_input(scanner);
            } else {
                create_account();
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
                    login(account_id, entered_account_id);
                }
            }
        }
        "q" => {
            // Exit the application
            scanner.clear();
            println!("Shutting down.");
            return ApplicationStatus::Complete;
        }
        _other => {
            eprintln!("Unexpected input: {}", _other);
        }
    }
    scanner.clear();
    ApplicationStatus::Continue
}

fn handle_command_logged_in(scanner: &mut Scanner, account_id: &String) -> ApplicationStatus {
    match scanner.next::<String>().as_str() {
        "b" => {
            //Submit a buy order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationStatus::Continue;
            }
            let ticker = scanner.next::<Ticker>();
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationStatus::Continue;
            }
            let price = scanner.next::<CentCount>();
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationStatus::Continue;
            }
            let quantity = scanner.next::<Quantity>();
            if !scanner.is_empty() {
                eprintln!("Unexpected input: ");
                print_remaining_input(scanner);
                return ApplicationStatus::Continue;
            }
            submit_buy(account_id, &ticker, &price, &quantity);
        }
        "s" => {
            //Submit a sell order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
            }
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
            println!("Shutting down.");
            return ApplicationStatus::Complete;
        }
        _other => {
            eprintln!("Unexpected input: {}", _other);
            scanner.clear();
        }
    }
    scanner.clear();
    ApplicationStatus::Continue
}
