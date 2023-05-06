use super::{Req, Crud};
use crate::Global;
use lib::GResult;
use std::sync::Arc;

pub async fn handler(Req { crud, .. }: &Req, global: &Arc<Global>) -> GResult<String> {
    match crud {
        Crud::Read => {
            let matcher = global.matcher.read().await;
            Ok(serde_json::to_string(&matcher.get_stats())?)
        }
        _ => Err(Box::from(format!("Can not {crud:?} market."))),
    }
}
