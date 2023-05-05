use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long)]
    coordinator: SocketAddr,
}

const HEADER_TEXT: &str =r"
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
                 STONKS ONLY GO UP - Warren Buffet
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

    println!("Actions:");
    println!("  c                Create a new account");
    println!("  l <account_id>   Login with your Account ID");

    loop {
        todo!()
    }

}

#[allow(dead_code)]
mod scanner {
    use std::collections::{HashSet, VecDeque};
    use std::{any::type_name, io::stdin, str::FromStr};

    pub struct Scanner {
        tokens: VecDeque<String>,
        delimiters: Option<HashSet<char>>,
    }
    impl Scanner {
        pub fn new() -> Self {
            Self {
                tokens: VecDeque::new(),
                delimiters: None,
            }
        }

        pub fn with_delimiters(delimiters: &[char]) -> Self {
            Self {
                tokens: VecDeque::new(),
                delimiters: Some(delimiters.iter().copied().collect()),
            }
        }

        pub fn next<T: FromStr>(&mut self) -> T {
            let token = loop {
                let front = self.tokens.pop_front();
                if let Some(token) = front {
                    break token;
                }
                self.receive_input();
            };
            token
                .parse::<T>()
                .unwrap_or_else(|_| panic!("input {} isn't a {}", token, type_name::<T>()))
        }

        pub fn next_line(&mut self) -> String {
            assert!(self.tokens.is_empty(), "You have unprocessed token");
            let mut line = String::new();
            stdin().read_line(&mut line).expect("Failed to read.");
            line.pop();
            line
        }

        fn receive_input(&mut self) {
            let mut line = String::new();
            stdin().read_line(&mut line).expect("Failed to read.");
            if let Some(delimiters) = &self.delimiters {
                for token in line.split(|c| delimiters.contains(&c)) {
                    self.tokens.push_back(token.to_string());
                }
            } else {
                for token in line.split_whitespace() {
                    self.tokens.push_back(token.to_string());
                }
            }
        }
    }
}
#[allow(unused_imports)]
use scanner::*;
