use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type NodeID = usize;
pub type CentCount = u64;
pub type Ticker = String;
pub type Quantity = u64;

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

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct UserID {
    pub id: usize,
    pub node_id: NodeID,
}
