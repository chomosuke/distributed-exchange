//! responsibility
//! Add order & cancel order comes in

use std::{
    cmp::min,
    collections::{BTreeMap, HashMap, VecDeque},
};

use lib::interfaces::{
    AllOrders, BuySell, CentCount, NodeID, OrderType, Quantity, QuantityPrice, Ticker, UserID,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub order_type: OrderType,
    pub ticker: Ticker,
    pub user_id: UserID,
    pub quantity: Quantity,
    pub price: CentCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub quantity: Quantity,
    pub price: CentCount,
    pub ticker: Ticker,
    pub buyer_id: UserID,
    pub seller_id: UserID,
}

pub struct Matcher {
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

    pub fn get_stats(&self) -> AllOrders {
        // TODO: Add comment to make this more readable
        let mut all_orders = HashMap::new();
        let self_buys_sells = [&self.buys, &self.sells];
        for (is_buy_sell, orders) in self_buys_sells.into_iter().enumerate() {
            for (ticker, price_quantity) in orders {
                let stats = all_orders.entry(ticker.to_owned()).or_insert(BuySell {
                    buy: Vec::new(),
                    sell: Vec::new(),
                });
                let stats_buys_sells = [&mut stats.buy, &mut stats.sell];
                for (&price, quantity) in price_quantity
                    .iter()
                    .map(|(price, quantity)| (price, quantity.iter().map(|(_, q)| q).sum()))
                {
                    stats_buys_sells[is_buy_sell].push(QuantityPrice { price, quantity })
                }
            }
        }
        AllOrders(all_orders)
    }

    #[allow(dead_code)] // TODO: investigate if this is needed
    pub fn deduct_order(
        &mut self,
        Order {
            order_type,
            ticker,
            user_id,
            quantity,
            price,
        }: Order,
    ) -> Result<(), Quantity> {
        let mut to_deduct = quantity;
        let existing_orders = match order_type {
            OrderType::Buy => &mut self.buys,
            OrderType::Sell => &mut self.sells,
        }
        .get_mut(&ticker)
        .ok_or(0_u64)?
        .get_mut(&price)
        .ok_or(0_u64)?;
        for (_, quantity) in existing_orders
            .iter_mut()
            .filter(|(other_user, _)| &user_id == other_user)
        {
            let deductable = min(to_deduct, *quantity);
            to_deduct -= deductable;
            *quantity -= deductable;
        }
        Ok(())
    }

    pub fn add_order(
        &mut self,
        Order {
            order_type,
            ticker,
            user_id,
            quantity: mut to_deduct,
            price,
        }: Order,
    ) -> (Order, Vec<Trade>) {
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
            for (other_user, quantity) in existing_orders.iter_mut().filter(|(other_user, _)| {
                user_id.node_id == self.this_id || other_user.node_id == self.this_id
            }) {
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

        let remaining_order = Order {
            order_type,
            ticker,
            user_id,
            quantity: to_deduct,
            price,
        };
        if remaining_order.quantity != 0 {
            match remaining_order.order_type {
                OrderType::Buy => &mut self.buys,
                OrderType::Sell => &mut self.sells,
            }
            .entry(remaining_order.ticker.clone())
            .or_default()
            .entry(remaining_order.price)
            .or_default()
            .push_back((remaining_order.user_id.clone(), remaining_order.quantity));
        }
        (remaining_order, proposed_trades)
    }
}
