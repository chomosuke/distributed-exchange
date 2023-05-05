//! format:
//! file name = UserID.id
//! file content = serde_json::to_string(Account)
//! file name = 'state'
//! file content = serde_json::to_string(StateFile)

use std::{collections::HashMap, error::Error, fs::read_to_string};

use serde::{Deserialize, Serialize};
use tokio::fs;

pub struct State {
    id: NodeID,
    accounts: Vec<Account>,
    per_dir: String,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    id: NodeID,
    account_nums: usize,
}

impl State {
    pub fn new(id: NodeID, per_dir: String) -> Self {
        Self {
            id,
            accounts: Vec::new(),
            per_dir,
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
            per_dir,
        })
    }

    pub fn get_id(&self) -> NodeID {
        self.id
    }

    pub async fn create_account(&mut self) -> Result<usize, Box<dyn Error>> {
        let id = self.accounts.len();
        self.accounts
            .push(Account::new(format!("{}/{id}", self.per_dir)).await?);
        Ok(id)
    }

    pub fn get_account_mut(&mut self, id: usize) -> &mut Account {
        &mut self.accounts[id]
    }
}

pub type NodeID = usize;
pub type CentCount = u64;
pub type Ticker = String;
pub type Quantity = u64;

#[derive(Debug, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}
pub struct Order {
    pub order_type: OrderType,
    pub ticker: Ticker,
    pub quantity: Quantity,
    pub price: CentCount,
}

/// need to tell matcher seperately
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
    async fn new(path: String) -> Result<Self, Box<dyn Error>> {
        let s = Self {
            path,
            balance: 0,
            portfolio: HashMap::new(),
            buys: HashMap::new(),
            sells: HashMap::new(),
        };
        s.update_file().await?;
        Ok(s)
    }

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

    pub async fn add_stock(&mut self, t: Ticker, q: Quantity) -> Result<(), Box<dyn Error>> {
        *self.portfolio.entry(t).or_default() += q;
        self.update_file().await
    }

    pub async fn deduct_stock(
        &mut self,
        t: Ticker,
        q: Quantity,
    ) -> Result<Quantity, Box<dyn Error>> {
        let current = self.portfolio.entry(t).or_default();
        let deducted = (*current).min(q);
        *current -= deducted;
        self.update_file().await?;
        Ok(deducted)
    }

    pub async fn add_order(
        &mut self,
        Order {
            order_type,
            ticker,
            quantity,
            price,
        }: Order,
    ) -> Result<(), Box<dyn Error>> {
        let orders = match order_type {
            OrderType::Buy => &mut self.buys,
            OrderType::Sell => &mut self.sells,
        };

        *orders.entry(ticker).or_default().entry(price).or_default() += quantity;
        self.update_file().await
    }

    /// can come from trade request or cancel order
    pub async fn deduct_order(
        &mut self,
        Order {
            order_type,
            ticker,
            quantity,
            price,
        }: Order,
    ) -> Result<Quantity, Box<dyn Error>> {
        let orders = match order_type {
            OrderType::Buy => &mut self.buys,
            OrderType::Sell => &mut self.sells,
        };

        let current = orders.entry(ticker).or_default().entry(price).or_default();
        let deducted = (*current).min(quantity);
        *current -= deducted;
        self.update_file().await?;
        Ok(deducted)
    }
}
