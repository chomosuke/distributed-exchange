use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct AllOrders(pub HashMap<String, BuySell>);

#[derive(Serialize, Deserialize)]
pub struct BuySell {
    pub buy: Vec<QuantityPrice>,
    pub sell: Vec<QuantityPrice>,
}

#[derive(Serialize, Deserialize)]
pub struct QuantityPrice {
    pub quantity: u64,
    pub price: u64,
}
