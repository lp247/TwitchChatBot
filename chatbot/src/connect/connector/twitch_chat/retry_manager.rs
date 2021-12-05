use crate::connect::error::ConnectorError;
use futures_retry::{ErrorHandler, RetryPolicy};
use std::time::Duration;

pub struct ExponentialRetryManager {
    init_wait_time: u64,
    max_num_attempts: usize,
}

impl ExponentialRetryManager {
    pub fn new(init_wait_time: Option<u64>, max_num_attempts: Option<usize>) -> Self {
        Self {
            init_wait_time: init_wait_time.unwrap_or(1),
            max_num_attempts: max_num_attempts.unwrap_or(3),
        }
    }
}

impl ErrorHandler<ConnectorError> for ExponentialRetryManager {
    type OutError = ConnectorError;

    fn handle(&mut self, attempt: usize, err: ConnectorError) -> RetryPolicy<Self::OutError> {
        if attempt > self.max_num_attempts {
            RetryPolicy::ForwardError(err)
        } else {
            // attempt should never be so big that it does not fit into u32
            // (try_into().unwrap())
            RetryPolicy::WaitRetry(Duration::from_secs(
                self.init_wait_time * 2_u64.pow(attempt.try_into().unwrap()),
            ))
        }
    }
    fn ok(&mut self, _attempt: usize) {}
}
