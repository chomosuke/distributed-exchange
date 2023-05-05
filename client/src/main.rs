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

const HEADER_TEXT: &str = r"
==================================================================================================================================================================
$$$$$$$\  $$\            $$\               $$\ $$\                  $$\                    $$\       $$\      $$\                     $$\                 $$\
$$  __$$\ \__|           $$ |              \__|$$ |                 $$ |                   $$ |      $$$\    $$$ |                    $$ |                $$ |
$$ |  $$ |$$\  $$$$$$$\$$$$$$\    $$$$$$\  $$\ $$$$$$$\  $$\   $$\$$$$$$\   $$$$$$\   $$$$$$$ |      $$$$\  $$$$ | $$$$$$\   $$$$$$\  $$ |  $$\  $$$$$$\$$$$$$\
$$ |  $$ |$$ |$$  _____\_$$  _|  $$  __$$\ $$ |$$  __$$\ $$ |  $$ \_$$  _| $$  __$$\ $$  __$$ |      $$\$$\$$ $$ | \____$$\ $$  __$$\ $$ | $$  |$$  __$$\_$$  _|
$$ |  $$ |$$ |\$$$$$$\   $$ |    $$ |  \__|$$ |$$ |  $$ |$$ |  $$ | $$ |   $$$$$$$$ |$$ /  $$ |      $$ \$$$  $$ | $$$$$$$ |$$ |  \__|$$$$$$  / $$$$$$$$ |$$ |
$$ |  $$ |$$ | \____$$\  $$ |$$\ $$ |      $$ |$$ |  $$ |$$ |  $$ | $$ |$$\$$   ____|$$ |  $$ |      $$ |\$  /$$ |$$  __$$ |$$ |      $$  _$$<  $$   ____|$$ |$$\
$$$$$$$  |$$ |$$$$$$$  | \$$$$  |$$ |      $$ |$$$$$$$  |\$$$$$$  | \$$$$  \$$$$$$$\ \$$$$$$$ |      $$ | \_/ $$ |\$$$$$$$ |$$ |      $$ | \$$\ \$$$$$$$\ \$$$$  |
\_______/ \__|\_______/   \____/ \__|      \__|\_______/  \______/   \____/ \_______| \_______|      \__|     \__| \_______|\__|      \__|  \__| \_______| \____/
==================================================================================================================================================================
                 STONKS ONLY GO UP - Warren Buffet, probably
==================================================================================================================================================================";

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

    println!("Choose an action:");
    println!("  c                Create a new account");
    println!("  l <account_id>   Login with your Account ID");
    println!("  q                Exit the application");

    let mut scanner: Scanner = Scanner::new();

    loop {
        match scanner.next::<String>().as_str() {
            "c" => { //Create a new account
                if !scanner.is_empty() {
                    eprintln!("Unexpected input after c: {}", scanner.next_line());
                    scanner.clear();
                }
                else {
                    create_account()
                }
            }
            "l" => { //Login with your Account ID
                if scanner.is_empty() {
                    eprintln!("Invalid input: Expected <account_id>");
                } else {
                    let account_id = scanner.next::<String>().clone();

                    if !scanner.is_empty() {
                        eprintln!("Unexpected input after account_id: ");
                        while !scanner.is_empty() {
                            eprint!("{}", scanner.next::<String>());
                        }
                        eprint!("\n");

                        scanner.clear();
                    } else {
                        login(account_id.to_string())
                    }
                }
            }
            "q" => { // Exit
                scanner.clear();
                println!("Shutting down.");
                break;
            }
            _other => {
                eprintln!("Unexpected input: {}", _other);
                scanner.clear();
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
