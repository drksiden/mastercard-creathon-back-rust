// Metrics utilities for performance tracking
// Can be extended with Prometheus or other metrics libraries

use std::time::Duration;

pub struct QueryMetrics {
    pub execution_time: Duration,
    pub row_count: usize,
}

impl QueryMetrics {
    pub fn new(execution_time: Duration, row_count: usize) -> Self {
        Self {
            execution_time,
            row_count,
        }
    }
}

