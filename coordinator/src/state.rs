use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio::{
    fs,
    sync::{mpsc::UnboundedSender, RwLock},
};

use crate::handlers;

#[derive(Serialize, Deserialize)]
pub struct NodeRecord {
    pub address: SocketAddr,

    #[serde(skip)]
    pub sender: Option<UnboundedSender<handlers::node::Message>>,
}

pub struct State {
    pub node_records: RwLock<NodeRecords>,
    pub account_nums: RwLock<AccountNums>,
}

impl State {
    pub async fn new_or_restore(per_dir: String) -> Self {
        match (
            NodeRecords::restore(per_dir.clone()).await,
            AccountNums::restore(per_dir.clone()).await,
        ) {
            (Some(n), Some(a)) => Self {
                node_records: RwLock::new(n),
                account_nums: RwLock::new(a),
            },
            _ => Self {
                node_records: RwLock::new(NodeRecords {
                    records: Vec::new(),
                    per_dir: per_dir.clone(),
                }),
                account_nums: RwLock::new(AccountNums {
                    nums: Vec::new(),
                    per_dir,
                }),
            },
        }
    }
}

pub struct NodeRecords {
    records: Vec<NodeRecord>,
    per_dir: String,
}

impl NodeRecords {
    async fn restore(per_dir: String) -> Option<Self> {
        let records: Vec<NodeRecord> = serde_json::from_str(
            &fs::read_to_string(format!("{per_dir}/node_records"))
                .await
                .ok()?,
        )
        .ok()?;
        Some(Self { records, per_dir })
    }

    async fn update_file(&mut self) {
        fs::write(
            format!("{}/node_records", self.per_dir),
            &serde_json::to_string(&self.records).unwrap(),
        )
        .await
        .expect("can't write to path");
    }

    pub async fn add_record(&mut self, node: NodeRecord) {
        self.records.push(node);
        self.update_file().await;
    }

    pub async fn set_record(&mut self, id: usize, node: NodeRecord) {
        self.records[id] = node;
        self.update_file().await;
    }

    pub fn get_records(&self) -> &Vec<NodeRecord> {
        &self.records
    }
}

pub struct AccountNums {
    nums: Vec<u64>,
    per_dir: String,
}

impl AccountNums {
    async fn restore(per_dir: String) -> Option<Self> {
        let nums: Vec<u64> = serde_json::from_str(
            &fs::read_to_string(format!("{per_dir}/account_nums"))
                .await
                .ok()?,
        )
        .ok()?;
        Some(Self { nums, per_dir })
    }

    async fn update_file(&mut self) {
        fs::write(
            format!("{}/account_nums", self.per_dir),
            &serde_json::to_string(&self.nums).unwrap(),
        )
        .await
        .expect("can't write to path");
    }

    pub async fn add_num(&mut self, node: u64) {
        self.nums.push(node);
        self.update_file().await;
    }

    pub async fn set_num(&mut self, id: usize, node: u64) {
        self.nums[id] = node;
        self.update_file().await;
    }

    pub fn get_nums(&self) -> &Vec<u64> {
        &self.nums
    }
}
