use std::{collections::{BTreeMap, HashMap, VecDeque}, hash::Hash};

type Ticker = String;
type DollarAmount = i32;
type Quantity = i32;
type UserId = String;

struct TickerOrders {
    buy: BTreeMap<DollarAmount, VecDeque<Order>>,
    sell: BTreeMap<DollarAmount, VecDeque<Order>>,
}

struct UserOrders {
    buy: HashMap<(Ticker, DollarAmount), Quantity>,
    sell: HashMap<(Ticker, DollarAmount), Quantity>,
}

enum OrderType {
    BUY,
    SELL
}

struct Order {
    order_type: OrderType,
    ticker: Ticker,
    user_id: UserId,
    quantity: Quantity,
    price: DollarAmount,
}

struct Wallet {
    user_id: UserId,
    cash_balance: DollarAmount,
    portfolio: HashMap<Ticker, Quantity>,
}

struct Trade {
    quantity: Quantity,
    price: DollarAmount,
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

        todo!()
        // match match match but only when one of them is an account you own
        // Not matched, add order
        // Need to keep user_orders in sync

        // if !self.ticker_orders.contains_key(&order.ticker) {
        //     self.ticker_orders.insert(, v)
        // }

        // let mut ticker_orders = self.ticker_orders.get_mut(&order.ticker)

        // match order.order_type {
        //     OrderType::BUY => {
        //         let proposed_trades: Vec<Trade> = Vec<Trade>::new();
        //         while {
        //             if let Some(entry) = self.user_orders.first_entry() {
        //                 entry.get().price < order.price
        //             } else {
        //                 false
        //             }
        //         } {
        //             // modify the sell_order's quantity. If quantity = 0, delete the sell_order
        //             let new_trade: Trade = Trade {
        //                 quantity: Quantity::min(order.quantity, entry.quantity),
        //                 price: Price::entry.get().price,
        //                 ticker: order.ticker,
        //                 buyer_id: order.user_id,
        //                 seller_id: entry.user_id,
        //             }



        //             proposed_trades.push(Trade::new());

        //             if order.quantity == 0 {
        //                 proposed_trades
        //             }

        //             // send trade offer to the seller
        //             // await
        //             // if successful, return
        //             // if failed,
        //             //   if order was deleted, recreate it
        //             //   if order was not deleted, add balance to it.

        //         }
        //         proposed_trades
        //     }
        //     OrderType::SELL => {
        //         if let Some(mut entry) = buy_map.first_entry() {
        //             if entry.get_mut().price > order.price {
        //                 // send trade offer to the buyer
        //                 // deduct from the seller's stock balance
        //             }
        //         }
        //     }
        // }
    }
}


    // // Initialise local accounts
    // let accounts: HashMap<UserId, Wallet> = HashMap::new();

    // // Initialise local market representation
    // let buy_orders: HashMap<Ticker, BTreeMap<DollarAmount, Order>> = HashMap::new();
    // let sell_orders: HashMap<Ticker, BTreeMap<DollarAmount, Order>> = HashMap::new();

    // loop {
    //     todo!()
    // }