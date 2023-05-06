//! format:
//! file name = UserID.id
//! file content = serde_json::to_string(Account)
//! file name = 'state'
//! file content = serde_json::to_string(StateFile)

use std::{collections::HashMap, fs::read_to_string};

use lib::GResult;
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock};

pub struct State {
    id: NodeID,
    accounts: HashMap<usize, RwLock<Account>>,
    next_account_id: usize,
    per_dir: String,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    id: NodeID,
    next_account_id: usize,
}

impl State {
    pub fn new(id: NodeID, per_dir: String) -> Self {
        Self {
            id,
            accounts: HashMap::new(),
            per_dir,
            next_account_id: 0,
        }
    }

    pub async fn restore(per_dir: String) -> Option<Self> {
        let state_file: StateFile =
            serde_json::from_str(&read_to_string(format!("{per_dir}/state")).ok()?).ok()?;
        let mut accounts = HashMap::new();
        for i in 0..state_file.next_account_id {
            if let Some(account) = Account::restore(format!("{per_dir}/{i}")).await {
                accounts.insert(i, RwLock::new(account));
            }
        }
        Some(Self {
            id: state_file.id,
            accounts,
            next_account_id: state_file.next_account_id,
            per_dir,
        })
    }

    pub fn get_id(&self) -> NodeID {
        self.id
    }

    pub async fn create_account(&mut self) -> GResult<usize> {
        let id = self.next_account_id;
        self.accounts
            .insert(id, RwLock::new(Account::new(format!("{}/{id}", self.per_dir)).await?));
        self.next_account_id += 1;
        Ok(id)
    }

    pub fn get_accounts(&self) -> &HashMap<usize, RwLock<Account>> {
        &self.accounts
    }

    pub fn get_accounts_mut(&mut self) -> &mut HashMap<usize, RwLock<Account>> {
        &mut self.accounts
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
    async fn new(path: String) -> GResult<Self> {
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

    async fn update_file(&self) -> GResult<()> {
        fs::write(&self.path, &serde_json::to_string(self)?).await?;
        Ok(())
    }

    async fn restore(path: String) -> Option<Self> {
        serde_json::from_str(&read_to_string(path).ok()?).ok()?
    }

    pub async fn delete(&mut self) -> Result<(), String> {
        if self.balance != 0 {
            return Err(format!("Can't delete account, balance not zero: {}", self.balance as f64 / 100.0));
        }
        if self.portfolio.iter().any(|(_, q)| q != &0) {
            return Err(format!("Can't delete account, portfolio not empty: {:?}", self.portfolio));
        }
        if self.buys.iter().any(|(_, o)| o.iter().any(|(_, q)| q != &0)) {
            return Err(format!("Can't delete account, still have buy orders: {:?}", self.buys));
        }
        if self.sells.iter().any(|(_, o)| o.iter().any(|(_, q)| q != &0)) {
            return Err(format!("Can't delete account, still have sell orders: {:?}", self.sells));
        }
        fs::remove_file(&self.path).await.map_err(|e| format!("Internal server error {e}"))
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub async fn set_balance(&mut self, value: u64) -> GResult<()> {
        self.balance = value;
        self.update_file().await
    }

    pub fn get_portfolio(&self) -> &HashMap<Ticker, Quantity> {
        &self.portfolio
    }

    pub async fn add_stock(&mut self, t: Ticker, q: Quantity) -> GResult<()> {
        *self.portfolio.entry(t).or_default() += q;
        self.update_file().await
    }

    pub async fn deduct_stock(&mut self, t: Ticker, q: Quantity) -> GResult<Quantity> {
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
    ) -> GResult<()> {
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
    ) -> GResult<Quantity> {
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
