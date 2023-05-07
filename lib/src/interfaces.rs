use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, fmt};

pub type NodeID = usize;
pub type CentCount = u64;
pub type Ticker = String;
pub type Quantity = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct AllOrders(pub HashMap<String, BuySell>);

#[derive(Debug, Serialize, Deserialize)]
pub struct BuySell {
    pub buy: Vec<QuantityPrice>,
    pub sell: Vec<QuantityPrice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuantityPrice {
    pub quantity: u64,
    pub price: u64,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct UserID {
    pub id: usize,
    pub node_id: NodeID,
}
impl fmt::Display for UserID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.node_id, self.id)
    }
}

#[derive(Debug)]
pub struct InvalidUserIDError;

impl FromStr for UserID {
    type Err = InvalidUserIDError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (l, r) = s.split_once('.').ok_or(InvalidUserIDError)?;
        let new_node_id = l.parse::<usize>().map_err(|_| InvalidUserIDError)?;
        let new_id = r.parse::<usize>().map_err(|_| InvalidUserIDError)?;
        Ok (UserID {
            id: new_id,
            node_id: new_node_id
        })
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderReq {
    pub order_type: OrderType,
    pub ticker: Ticker,
    pub price: CentCount,
    pub quantity: Quantity,
}
