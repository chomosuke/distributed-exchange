//! format:
//! file name = UserID.id
//! file content = serde_json::to_string(Account)

use serde::{Serialize, Deserialize};

pub struct Accounts {
    accounts: Vec<Account>,
}

impl Accounts {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }
    pub fn restore(per_dir: String) -> Self {
        
    }
}

#[derive(Serialize, Deserialize)]
struct Account {
    
}
