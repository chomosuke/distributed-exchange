use std::{future::Future, time::Duration};

use async_trait::async_trait;
use tokio::time::timeout;

#[async_trait]
pub trait DeadLockDetect {
    async fn dl(self, msg: &str) -> Self::Output
    where
        Self: Future;
}

#[async_trait]
impl<F: Future + Send> DeadLockDetect for F {
    async fn dl(self, msg: &str) -> F::Output {
        timeout(Duration::new(5, 0), self)
            .await
            .unwrap_or_else(|_| panic!("Timeout! Future took longer than 5 second: {msg}"))
    }
}
