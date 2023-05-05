use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, VecDeque},
};

type Ticker = String;
type CentAmount = u64;
type Quantity = u64;
type UserId = String;

struct TickerOrders {
    buy: BTreeMap<CentAmount, VecDeque<Order>>,
    sell: BTreeMap<CentAmount, VecDeque<Order>>,
}

struct UserOrders {
    buy: HashMap<(Ticker, CentAmount), Quantity>,
    sell: HashMap<(Ticker, CentAmount), Quantity>,
}

enum OrderType {
    BUY,
    SELL,
}

struct Order {
    order_type: OrderType,
    ticker: Ticker,
    user_id: UserId,
    quantity: Quantity,
    price: CentAmount,
}

struct Wallet {
    user_id: UserId,
    cash_balance: CentAmount,
    portfolio: HashMap<Ticker, Quantity>,
}

struct Trade {
    quantity: Quantity,
    price: CentAmount,
    ticker: Ticker,
    buyer_id: UserId,
    seller_id: UserId,
}

struct Matcher {
    ticker_orders: HashMap<Ticker, TickerOrders>,
    user_orders: HashMap<UserId, UserOrders>,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            ticker_orders: HashMap::new(),
            user_orders: HashMap::new(),
        }
    }
    // {
    //     buy: BTreeMap<DollarAmount, VecDeque<Order>>::new(),
    //     sell: BTreeMap<DollarAmount, VecDeque<Order>>::new(),
    // },

    pub fn add_order(&mut self, order: Order) -> Vec<Trade> {
        let mut quantity = order.quantity;

        // match match match but only when one of them is an account you own
        // Not matched, add order
        // Need to keep user_orders in sync

        if !self.ticker_orders.contains_key(&order.ticker) {
            self.ticker_orders.insert(
                order.ticker.clone(),
                TickerOrders {
                    buy: BTreeMap::new(),
                    sell: BTreeMap::new(),
                },
            );
        }

        let ticker_orders = self.ticker_orders.get_mut(&order.ticker).unwrap();

        let existing_orders = match order.order_type {
            OrderType::BUY => &mut ticker_orders.sell,
            OrderType::SELL => &mut ticker_orders.buy,
        };

        let mut proposed_trades: Vec<Trade> = Vec::new();

        while {
            if let Some(entry) = existing_orders.first_entry() {
                entry.key() < &order.price && quantity != 0
            } else {
                false
            }
        } {
            let mut entry = existing_orders.first_entry().unwrap();
            let order_list = entry.get_mut();

            while !order_list.is_empty() && quantity != 0 {
                // modify the sell_order's quantity. If quantity = 0, delete the sell_order
                let matched_order = order_list.front_mut().unwrap();

                let new_trade: Trade = Trade {
                    quantity: min(quantity, matched_order.quantity),
                    price: matched_order.price,
                    ticker: order.ticker.clone(),
                    buyer_id: order.user_id.clone(),
                    seller_id: matched_order.user_id.clone(),
                };

                quantity -= new_trade.quantity;
                matched_order.quantity -= new_trade.quantity;

                proposed_trades.push(new_trade);

                if matched_order.quantity == 0 {
                    order_list.pop_front();
                }
            }
            if order_list.is_empty() {
                entry.remove_entry();
            }

            //TODO: Remove matched_order from UserOrders

            // send trade offer to the seller
            // await
            // if successful, return
            // if failed,
            //   if order was deleted, recreate it
            //   if order was not deleted, add balance to it.
        }
        proposed_trades
    }
}

// // Initialise local accounts
// let accounts: HashMap<UserId, Wallet> = HashMap::new();

// // Initialise local market representation
// let buy_orders: HashMap<Ticker, BTreeMap<DollarAmount, Order>> = HashMap::new();
// let sell_orders: HashMap<Ticker, BTreeMap<DollarAmount, Order>> = HashMap::new();