//! format:
//! file name = UserID.id
//! file content = serde_json::to_string(Account)
//! file name = 'state'
//! file content = serde_json::to_string(StateFile)

use std::{collections::HashMap, error::Error, fs::read_to_string, path::Path};

use serde::{Deserialize, Serialize};
use tokio::fs;

pub struct State {
    id: NodeID,
    accounts: Vec<Account>,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    id: NodeID,
    account_nums: usize,
}

impl State {
    pub fn new(id: NodeID) -> Self {
        Self {
            id,
            accounts: Vec::new(),
        }
    }

    pub async fn restore(per_dir: String) -> Option<Self> {
        let state_file: StateFile =
            serde_json::from_str(&read_to_string(format!("{per_dir}/state")).ok()?).ok()?;
        let mut accounts = Vec::new();
        for i in 0..state_file.account_nums {
            accounts.push(Account::restore(format!("{per_dir}/{i}")).await?);
        }
        Some(Self {
            id: state_file.id,
            accounts,
        })
    }

    pub fn get_id(&self) -> NodeID {
        self.id
    }
}

pub type NodeID = usize;
pub type CentCount = u64;
pub type Ticker = String;
pub type Quantity = u64;

pub enum OrderType {
    BUY,
    SELL,
}
pub struct Order {
    order_type: OrderType,
    ticker: Ticker,
    quantity: Quantity,
    price: CentCount,
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    #[serde(skip)]
    path: String,

    balance: CentCount,
    portfolio: HashMap<Ticker, Quantity>,
    buys: HashMap<Ticker, HashMap<CentCount, Quantity>>,
    sells: HashMap<Ticker, HashMap<CentCount, Quantity>>,
}

impl Account {
    async fn update_file(&self) -> Result<(), Box<dyn Error>> {
        fs::write(&self.path, &serde_json::to_string(self)?).await?;
        Ok(())
    }

    async fn restore(path: String) -> Option<Self> {
        serde_json::from_str(&read_to_string(path).ok()?).ok()?
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub async fn set_balance(&mut self, value: u64) -> Result<(), Box<dyn Error>> {
        self.balance = value;
        self.update_file().await
    }

    pub fn get_portfolio(&self) -> &HashMap<Ticker, Quantity> {
        &self.portfolio
    }

    pub async fn set_portfolio(&mut self, t: Ticker, q: Quantity) -> Result<(), Box<dyn Error>> {
        if q == 0 {
            self.portfolio.remove(&t);
        } else {
            self.portfolio.insert(t, q);
        }
        self.update_file().await
    }

    pub async fn add_order(&mut self, o: Order) -> Result<(), Box<dyn Error>> {
        let Order { order_type, ticker, quantity, price } = o;
        let orders = match order_type {
            OrderType::BUY => &mut self.buys,
            OrderType::SELL => &mut self.sells,
        };

        *orders.entry(ticker).or_default().entry(price).or_default() += quantity;
        self.update_file().await
    }

    pub async fn cancel_order(&mut self, o: Order) -> Result<Quantity, Box<dyn Error>> {
        let Order { order_type, ticker, quantity, price } = o;
        let orders = match order_type {
            OrderType::BUY => &mut self.buys,
            OrderType::SELL => &mut self.sells,
        };

        let q = orders.entry(ticker).or_default().entry(quantity).or_default();
        let deducted = (*q).min(quantity);
        *q -= deducted;
        self.update_file().await?;
        Ok(deducted)
    }
}
