use serde::Deserialize;
use std::{error::Error, str::FromStr, sync::Arc};

use super::ReadWriter;
use crate::Global;

#[derive(Deserialize)]
pub struct UserID {
    id: u64,
    node_id: u64,
}

pub enum FirstLine {
    CAccount,
    FindNode(UserID),
}

impl FromStr for FirstLine {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(user_id) = serde_json::from_str(s) {
            Ok(FirstLine::FindNode(user_id))
        } else if s == "C Account" {
            Ok(FirstLine::CAccount)
        } else {
            Err("Did not match first line for client".into())
        }
    }
}

pub async fn handler(
    first_line: FirstLine,
    rw: ReadWriter<'_>,
    global: Arc<Global>,
) -> Result<String, Box<dyn Error>> {
    
}
