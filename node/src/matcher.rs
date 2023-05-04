struct Matcher {
    tickers_orders: HashMap<Ticker, TickerOrders>,
    user_orders: HashMap<UserId, UserOrders>,
}

type Ticker = String;
type DollarAmount = i32;
type Quantity = i32;
type UserId = String;

struct TickerOrders {
    buy: BTreeMap<DollarAmount, VecDeque<Order>>,
    sell: BTreeMap<DollarAmount, VecDeque<Order>>,
}

struct UserOrders {
    buy: HashMap<(Ticker, DollorAmount), Quantity>,
    sell: HashMap<(Ticker, DollorAmount), Quantity>,
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

impl Matcher {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn add_order(order: Order) -> Vec<Trade> {
        // match match match but only when one of them is an account you own
        // Not matched, add order
        // Need to keep user_orders in sync
    }
}
