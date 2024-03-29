//! format:
//! file name = UserID.id
//! file content = serde_json::to_string(Account)
//! file name = 'state'
//! file content = serde_json::to_string(StateFile)

use crate::{
    handlers::node::{Offer, TradeID},
    matcher::{Order, Trade},
};
use lib::{
    interfaces::{
        AllOrders, BuySell, CentCount, NodeID, OrderReq, OrderType, Quantity, QuantityPrice,
        Ticker, UserID,
    },
    lock::DeadLockDetect,
    GResult,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::read_to_string};
use tokio::{fs, sync::RwLock};

pub struct State {
    id: NodeID,
    next_account_id: usize,
    next_trade_id: usize,
    pending_to_user: HashMap<TradeID, usize>,
    accounts: HashMap<usize, RwLock<Account>>,
    per_dir: String,
}

#[derive(Serialize, Deserialize)]
struct StateFile {
    id: NodeID,
    next_account_id: usize,
    next_trade_id: usize,
    pending_to_user: HashMap<TradeID, usize>,
}

impl State {
    pub fn new(id: NodeID, per_dir: String) -> Self {
        Self {
            id,
            next_account_id: 0,
            next_trade_id: 0,
            accounts: HashMap::new(),
            pending_to_user: HashMap::new(),
            per_dir,
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
            next_trade_id: state_file.next_trade_id,
            per_dir,
            pending_to_user: HashMap::new(),
        })
    }

    async fn update_file(&mut self) -> GResult<()> {
        fs::write(
            format!("{}/state", self.per_dir),
            &serde_json::to_string(&StateFile {
                id: self.id,
                next_account_id: self.next_account_id,
                next_trade_id: self.next_trade_id,
                pending_to_user: self.pending_to_user.clone(),
            })?,
        )
        .await?;
        Ok(())
    }

    pub fn get_id(&self) -> NodeID {
        self.id
    }

    pub async fn create_account(&mut self) -> GResult<usize> {
        let id = self.next_account_id;
        self.accounts.insert(
            id,
            RwLock::new(
                Account::new(
                    format!("{}/{id}", self.per_dir),
                    UserID {
                        id,
                        node_id: self.id,
                    },
                )
                .await?,
            ),
        );
        self.next_account_id += 1;
        self.update_file().await?;
        Ok(id)
    }

    pub fn get_accounts(&self) -> &HashMap<usize, RwLock<Account>> {
        &self.accounts
    }

    pub fn remove_account(&mut self, id: usize) -> Option<RwLock<Account>> {
        self.accounts.remove(&id)
    }

    pub async fn process_matches(&mut self, matches: Vec<Trade>) -> GResult<Vec<(NodeID, Offer)>> {
        let mut offers = Vec::new();
        for trade in matches {
            if trade.buyer_id.node_id == self.id && trade.seller_id.node_id == self.id {
                // Both local, perform trade NOW
                let Trade {
                    quantity,
                    price,
                    ticker,
                    buyer_id,
                    seller_id,
                    buy_price,
                    sell_price,
                } = trade;
                let buyer = &mut self
                    .accounts
                    .get(&buyer_id.id)
                    .expect("Matcher gave invalid UserID")
                    .write()
                    .dl("st134")
                    .await;
                let seller = &mut self
                    .accounts
                    .get(&seller_id.id)
                    .expect("Matcher gave invalid UserID")
                    .write()
                    .dl("st141")
                    .await;
                let new_buyer_balance = buyer.get_balance() - quantity * price;
                buyer.set_balance(new_buyer_balance).await?;
                assert_eq!(
                    quantity,
                    seller.deduct_stock(ticker.clone(), quantity).await?
                );
                let new_seller_balance = seller.get_balance() + quantity * price;
                seller.set_balance(new_seller_balance).await?;
                buyer.add_stock(ticker.clone(), quantity).await?;

                let deducted = seller
                    .deduct_order(OrderReq {
                        order_type: OrderType::Sell,
                        ticker: ticker.clone(),
                        price: sell_price,
                        quantity,
                    })
                    .await?;
                assert_eq!(deducted, quantity);
                let deducted = buyer
                    .deduct_order(OrderReq {
                        order_type: OrderType::Buy,
                        ticker: ticker.clone(),
                        price: buy_price,
                        quantity,
                    })
                    .await?;
                assert_eq!(deducted, quantity);
            } else {
                // One of them remote
                let (mut local, remote) = if trade.buyer_id.node_id == self.id {
                    (
                        self.accounts[&trade.buyer_id.id].write().dl("st156").await,
                        trade.seller_id,
                    )
                } else {
                    assert_eq!(
                        trade.seller_id.node_id, self.id,
                        "Matcher error, both buyer and seller are not local"
                    );
                    (
                        self.accounts[&trade.seller_id.id].write().dl("st165").await,
                        trade.buyer_id,
                    )
                };
                let trade_id = self.next_trade_id;
                self.next_trade_id += 1;
                local.add_pending(trade_id, trade.clone()).await?;

                // record which TradeID belong to which user
                self.pending_to_user.insert(trade_id, local.id.id);

                // return the offer to be sent
                offers.push((
                    remote.node_id,
                    Offer {
                        id: trade_id,
                        trade,
                    },
                ))
            }
        }
        self.update_file().await?;
        Ok(offers)
    }

    pub async fn commit_pending(&mut self, trade_id: TradeID) -> GResult<()> {
        let user_id = self
            .pending_to_user
            .remove(&trade_id)
            .expect("Non existent trade_id");
        let account = &self.accounts[&user_id];
        account
            .write()
            .dl("st193")
            .await
            .commit_pending(trade_id)
            .await?;
        self.update_file().await
    }

    pub async fn abort_pending(&mut self, trade_id: TradeID) -> GResult<Order> {
        let user_id = self
            .pending_to_user
            .remove(&trade_id)
            .expect("Non existent trade_id");
        let account = &self.accounts[&user_id];
        let order = account
            .write()
            .dl("st200")
            .await
            .abort_pending(trade_id)
            .await?;
        self.update_file().await?;
        Ok(order)
    }
}

/// need to tell matcher seperately
#[derive(Serialize, Deserialize)]
pub struct Account {
    #[serde(skip)]
    path: String,

    id: UserID,
    balance: CentCount,
    portfolio: HashMap<Ticker, Quantity>,
    buys: HashMap<Ticker, HashMap<CentCount, Quantity>>,
    sells: HashMap<Ticker, HashMap<CentCount, Quantity>>,
    pending: HashMap<TradeID, Trade>,
}

impl Account {
    async fn new(path: String, id: UserID) -> GResult<Self> {
        let s = Self {
            id,
            path,
            balance: 0,
            portfolio: HashMap::new(),
            buys: HashMap::new(),
            sells: HashMap::new(),
            pending: HashMap::new(),
        };
        s.update_file().await?;
        Ok(s)
    }

    async fn update_file(&self) -> GResult<()> {
        fs::write(&self.path, &serde_json::to_string(self)?)
            .await
            .expect(&self.path);
        Ok(())
    }

    async fn restore(path: String) -> Option<Self> {
        let mut s: Self = serde_json::from_str(&read_to_string(path.clone()).ok()?).ok()?;
        s.path = path;
        Some(s)
    }

    pub async fn delete(&mut self) -> Result<(), String> {
        if self.balance != 0 {
            return Err(format!(
                "Can't delete account, balance not zero: {}",
                self.balance as f64 / 100.0
            ));
        }
        if self.portfolio.iter().any(|(_, q)| q != &0) {
            return Err(format!(
                "Can't delete account, portfolio not empty: {:?}",
                self.portfolio
            ));
        }
        if self
            .buys
            .iter()
            .any(|(_, o)| o.iter().any(|(_, q)| q != &0))
        {
            return Err(format!(
                "Can't delete account, still have buy orders: {:?}",
                self.buys
            ));
        }
        if self
            .sells
            .iter()
            .any(|(_, o)| o.iter().any(|(_, q)| q != &0))
        {
            return Err(format!(
                "Can't delete account, still have sell orders: {:?}",
                self.sells
            ));
        }
        fs::remove_file(&self.path)
            .await
            .map_err(|e| format!("Internal server error {e}"))
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub async fn set_balance(&mut self, value: u64) -> GResult<bool> {
        if value < self.get_buy_order_amount() {
            Ok(false)
        } else {
            self.balance = value;
            self.update_file().await?;
            Ok(true)
        }
    }

    pub fn get_portfolio(&self) -> &HashMap<Ticker, Quantity> {
        &self.portfolio
    }

    pub fn get_orders(&self) -> AllOrders {
        let mut all_orders = HashMap::new();
        let self_buys_sells = [&self.buys, &self.sells];
        for (is_buy_sell, orders) in self_buys_sells.into_iter().enumerate() {
            for (ticker, price_quantity) in orders {
                let stats = all_orders.entry(ticker.to_owned()).or_insert(BuySell {
                    buy: Vec::new(),
                    sell: Vec::new(),
                });
                let stats_buys_sells = [&mut stats.buy, &mut stats.sell];
                for (&price, &quantity) in price_quantity {
                    stats_buys_sells[is_buy_sell].push(QuantityPrice { price, quantity })
                }
            }
        }
        AllOrders(all_orders)
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

    pub fn get_buy_order_amount(&self) -> CentCount {
        self.buys
            .values()
            .map(|orders| {
                orders
                    .iter()
                    .map(|(price, quantity)| price * quantity)
                    .sum::<u64>()
            })
            .sum()
    }

    pub fn get_sell_order_quantity(&self, ticker: &Ticker) -> Quantity {
        match self.sells.get(ticker) {
            Some(s) => s,
            None => return 0,
        }
        .values()
        .sum()
    }

    /// Attempt to add order to the account
    pub async fn add_order(
        &mut self,
        OrderReq {
            order_type,
            ticker,
            quantity,
            price,
        }: OrderReq,
    ) -> GResult<bool> {
        // check if order can be added
        match order_type {
            OrderType::Buy => {
                if self.balance - self.get_buy_order_amount() < quantity * price {
                    // too many orders, not enough money
                    return Ok(false);
                }
            }
            OrderType::Sell => {
                if *self.portfolio.get(&ticker).unwrap_or(&0)
                    - self.get_sell_order_quantity(&ticker)
                    < quantity
                {
                    // too many orders, not enough stock
                    return Ok(false);
                }
            }
        }

        let orders = match order_type {
            OrderType::Buy => &mut self.buys,
            OrderType::Sell => &mut self.sells,
        };

        *orders.entry(ticker).or_default().entry(price).or_default() += quantity;
        self.update_file().await?;
        Ok(true)
    }

    /// can come from trade request or cancel order
    pub async fn deduct_order(
        &mut self,
        OrderReq {
            order_type,
            ticker,
            quantity,
            price,
        }: OrderReq,
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

    /// accept or reject a trade offer, modifying to account in case accepted
    /// Return order deducted if accepted
    pub async fn process_incoming_offer(&mut self, trade: Trade) -> GResult<Option<Order>> {
        let Trade {
            quantity,
            price,
            ticker,
            buyer_id,
            seller_id,
            buy_price,
            sell_price,
        } = trade;

        let order_price = if buyer_id == self.id {
            buy_price
        } else {
            sell_price
        };

        // check if enough orders left
        let current_orders = if buyer_id == self.id {
            &mut self.buys
        } else {
            &mut self.sells
        };
        let current_order_quantity = current_orders
            .entry(ticker.clone())
            .or_default()
            .entry(order_price)
            .or_default();
        if *current_order_quantity < quantity {
            println!("rejected order: {quantity} {current_order_quantity}");
            return Ok(None);
        }

        if buyer_id == self.id {
            let to_deduct = quantity * price;
            if self.balance < to_deduct {
                println!("rejected quantity: {} {to_deduct}", self.balance);
                return Ok(None);
            }
            // commit
            self.balance -= to_deduct;
            *self.portfolio.entry(ticker.clone()).or_default() += quantity;
        } else if seller_id == self.id {
            let current_quantity = self.portfolio.entry(ticker.clone()).or_default();
            if *current_quantity < quantity {
                println!("rejected stock: {quantity} {current_quantity}");
                return Ok(None);
            }
            // commit
            *current_quantity -= quantity;
            self.balance += quantity * price;
        } else {
            panic!("This trade doesn't belong to this user");
        }

        *current_order_quantity -= quantity;
        self.update_file().await?;
        Ok(Some(Order {
            quantity,
            order_type: if buyer_id == self.id {
                OrderType::Buy
            } else {
                OrderType::Sell
            },
            ticker,
            user_id: self.id,
            price: order_price,
        }))
    }

    /// this function assume the trade will succeed
    pub async fn add_pending(&mut self, trade_id: TradeID, trade: Trade) -> GResult<()> {
        assert!(
            self.pending.get(&trade_id).is_none(),
            "duplicate trade id??"
        );
        self.pending.insert(trade_id, trade.clone());
        let Trade {
            quantity,
            price,
            ticker,
            buyer_id,
            seller_id,
            buy_price,
            sell_price,
        } = trade.clone();
        if buyer_id == self.id {
            let to_deduct = quantity * price;
            assert!(
                to_deduct <= self.balance,
                "Invalid trade, not enough balance"
            );
            self.balance -= to_deduct;
        } else if seller_id == self.id {
            let current_quantity = self.portfolio.entry(ticker.clone()).or_default();
            assert!(
                quantity <= *current_quantity,
                "Invalid trade, not enough stock"
            );
            *current_quantity -= quantity;
        } else {
            panic!("This trade doesn't belong to this user");
        }

        // check if enough orders left
        let current_orders = if buyer_id == self.id {
            &mut self.buys
        } else {
            &mut self.sells
        };
        let current_order_quantity = current_orders
            .entry(ticker)
            .or_default()
            .entry(if buyer_id == self.id {
                buy_price
            } else {
                sell_price
            })
            .or_default();
        assert!(
            quantity <= *current_order_quantity,
            "Invalid trade, not enough order {trade:?}"
        );
        *current_order_quantity -= quantity;

        self.update_file().await
    }

    pub async fn commit_pending(&mut self, trade_id: TradeID) -> GResult<()> {
        let Trade {
            quantity,
            price,
            ticker,
            buyer_id,
            seller_id,
            ..
        } = self.pending.remove(&trade_id).expect("Invalid trade_id");
        if buyer_id == self.id {
            let current_quantity = self.portfolio.entry(ticker).or_default();
            *current_quantity += quantity;
        } else if seller_id == self.id {
            self.balance += quantity * price;
        } else {
            panic!("This trade doesn't belong to this user");
        }
        self.update_file().await
    }

    pub async fn abort_pending(&mut self, trade_id: TradeID) -> GResult<Order> {
        let Trade {
            quantity,
            price,
            ticker,
            buyer_id,
            seller_id,
            buy_price,
            sell_price,
        } = self.pending.remove(&trade_id).expect("Invalid trade_id");
        if buyer_id == self.id {
            self.balance += quantity * price;
        } else if seller_id == self.id {
            let current_quantity = self.portfolio.entry(ticker.clone()).or_default();
            *current_quantity += quantity;
        } else {
            panic!("This trade doesn't belong to this user");
        }

        // add orders back
        let (order_type, price) = if buyer_id == self.id {
            (OrderType::Buy, buy_price)
        } else {
            (OrderType::Sell, sell_price)
        };
        let current_orders = if buyer_id == self.id {
            &mut self.buys
        } else {
            &mut self.sells
        };
        let current_order_quantity = current_orders
            .entry(ticker.clone())
            .or_default()
            .entry(if buyer_id == self.id {
                buy_price
            } else {
                sell_price
            })
            .or_default();
        *current_order_quantity += quantity;

        self.update_file().await?;
        Ok(Order {
            price,
            user_id: self.id,
            order_type,
            quantity,
            ticker,
        })
    }
}
