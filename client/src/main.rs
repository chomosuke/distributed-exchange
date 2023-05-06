mod scanner;

use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[allow(unused_imports)]
use scanner::*;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,
}

enum ApplicationStatus {
    CONTINUE,
    COMPLETE
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

        if account_id.is_empty() { // not logged in
            println!("  c                Create a new account");
            println!("  l <account_id>   Login with your Account ID");
            println!("  q                Exit the application");

            match get_command_logged_out(scanner) {
                ApplicationStatus::COMPLETE => {break}
                ApplicationStatus::CONTINUE => {}
            }

        } else { // logged in
            println!("Choose an action:");
            println!("  b <ticker> <price> <quantity>  Submit a buy order");
            println!("  s <ticker> <price> <quantity>  Submit a sell order");
            println!("  c <ticker> <price> <quantity>  Cancel an order");
            println!("  o                              See your submitted orders");
            println!("  a                              See current account details");
            println!("  p                              See current stock prices");
            println!("  q                              Exit the application");
            // Delete account
            // Going IPO
            // Cancel order buy or sell
            // deposite and withdraw money

            match get_command_logged_in(scanner, account_id) {
                ApplicationStatus::COMPLETE => {break}
                ApplicationStatus::CONTINUE => {}
            }

        }


    }
}

fn create_account() {
    todo!();
}

fn login(account_id: String) {
    todo!()
}

fn submit_buy(account_id: String, ticker: String, price: String, quantity: String) {
    todo!()
}

fn submit_sell(account_id: String, ticker: String, price: String, quantity: String) {
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

fn print_remaining_input(mut scanner: Scanner) {
    while !scanner.is_empty() {
        eprint!("{} ", scanner.next::<String>());
    }
    eprintln!();
}

fn get_command_logged_out(mut scanner: Scanner) -> ApplicationStatus {
    match scanner.next::<String>().as_str() {
        "c" => { //Create a new account
            if !scanner.is_empty() {
                eprintln!("Unexpected input after c: ");
                print_remaining_input(scanner);
            }
            else {
                create_account();
            }
        }
        "l" => { //Login with your Account ID
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <account_id>");
            } else {
                let account_id = scanner.next::<String>().clone();

                if !scanner.is_empty() {
                    eprint!("Unexpected input after account_id: ");
                    print_remaining_input(scanner);
                } else {
                    login(account_id.to_string());
                }
            }
        }
        "q" => { // Exit the application
            scanner.clear();
            println!("Shutting down.");
            return ApplicationStatus::COMPLETE;
        }
        _other => {
            eprintln!("Unexpected input: {}", _other);
        }
    }
    scanner.clear();
    ApplicationStatus::CONTINUE
}

fn get_command_logged_in(mut scanner: Scanner, account_id: String) -> ApplicationStatus {
    match scanner.next::<String>().as_str() {
        "b" => { //Submit a buy order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationStatus::CONTINUE;
            }
            let ticker = scanner.next::<String>();
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationStatus::CONTINUE;
            }
            let price = scanner.next::<String>();
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
                return ApplicationStatus::CONTINUE;
            }
            let quantity = scanner.next::<String>();
            if !scanner.is_empty() {
                eprintln!("Unexpected input: ");
                print_remaining_input(scanner);
                return ApplicationStatus::CONTINUE;
            }
            submit_buy(account_id, ticker, price, quantity);
        }
        "s" => { //Submit a sell order
            if scanner.is_empty() {
                eprintln!("Invalid input: Expected <ticker> <price> <quantity>");
            }
        }
        "c" => { //Cancel an order
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
        "q" => { // Exit the application
            scanner.clear();
            println!("Shutting down.");
            return ApplicationStatus::COMPLETE;
        }
        _other => {
            eprintln!("Unexpected input: {}", _other);
            scanner.clear();
        }
    }
    scanner.clear();
    ApplicationStatus::CONTINUE
}
