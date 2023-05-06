use std::{future::Future, time::Duration};

use tokio::time::{timeout, Timeout};

pub trait DeadLockDetect<F: Future> {
    fn dl(future: F) -> Timeout<F>;
}

impl<F: Future> DeadLockDetect<F> for F {
    fn dl(future: F) -> Timeout<F> {
        timeout(Duration::new(5, 0), future)
    }
}
