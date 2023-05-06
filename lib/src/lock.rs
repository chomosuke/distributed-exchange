use std::{future::Future, time::Duration};

use async_trait::async_trait;
use tokio::time::timeout;

#[async_trait]
pub trait DeadLockDetect {
    async fn dl(self) -> Self::Output
    where
        Self: Future;
}

#[async_trait]
impl<F: Future + Send> DeadLockDetect for F {
    async fn dl(self) -> F::Output {
        timeout(Duration::new(5, 0), self)
            .await
            .expect("Timeout! Future took longer than 5 second")
    }
}
