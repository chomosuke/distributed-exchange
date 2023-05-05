//! responsibility
//! Add order & cancel order comes in

use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, VecDeque},
};

use crate::{
    handlers::client::UserID,
    state::{CentCount, NodeID, OrderType, Quantity, Ticker},
};

struct Order {
    order_type: OrderType,
    ticker: Ticker,
    user_id: UserID,
    quantity: Quantity,
    price: CentCount,
}

struct Trade {
    quantity: Quantity,
    price: CentCount,
    ticker: Ticker,
    buyer_id: UserID,
    seller_id: UserID,
}

struct Matcher {
    this_id: NodeID,
    buys: HashMap<Ticker, BTreeMap<CentCount, VecDeque<(UserID, Quantity)>>>,
    sells: HashMap<Ticker, BTreeMap<CentCount, VecDeque<(UserID, Quantity)>>>,
}

impl Matcher {
    pub fn new(this_id: NodeID) -> Self {
        Self {
            this_id,
            buys: HashMap::new(),
            sells: HashMap::new(),
        }
    }

    // pub fn deduct_order(&mut self, order: Order) -> Quantity {}

    pub fn add_order(
        &mut self,
        Order {
            order_type,
            ticker,
            user_id,
            quantity: mut to_deduct,
            price,
        }: Order,
    ) -> Vec<Trade> {
        let existing_orders = match order_type {
            OrderType::Buy => &mut self.sells,
            OrderType::Sell => &mut self.buys,
        }
        .entry(ticker.clone())
        .or_default();

        let mut proposed_trades: Vec<Trade> = Vec::new();

        // rust types slowing me down again
        let price_range: Box<dyn Iterator<Item = _>> = match order_type {
            OrderType::Buy => Box::from(existing_orders.range_mut(..=price)),
            OrderType::Sell => Box::from(existing_orders.range_mut(price..).rev()),
        };

        'outer: for (other_price, existing_orders) in price_range {
            for (other_user, quantity) in
                existing_orders.iter_mut().filter(|(other_user, _)| {
                    user_id.node_id == self.this_id || other_user.node_id == self.this_id
                })
            {
                let new_trade: Trade = Trade {
                    quantity: min(to_deduct, *quantity),
                    price: *other_price,
                    ticker: ticker.clone(),
                    buyer_id: user_id.clone(),
                    seller_id: other_user.clone(),
                };

                *quantity -= new_trade.quantity;
                to_deduct -= new_trade.quantity;

                proposed_trades.push(new_trade);

                if to_deduct == 0 {
                    break 'outer;
                }
            }
            // collect garbage
            while let Some((_, 0)) = existing_orders.front() {
                existing_orders.pop_front();
            }
        }

        // collect garbage
        while existing_orders.first_key_value().map(|(_, vd)| vd.len()) == Some(0) {
            existing_orders.pop_first();
        }

        proposed_trades
    }
}

