use std::{ops::AddAssign, time::Duration};

pub trait MetricType<T: Copy + Clone + Default + AddAssign> {
    fn register(&mut self, value: T);
    fn get(&self) -> T;
}

#[derive(Copy, Clone, Default)]
pub struct Counter {
    count: u32,
}

impl MetricType<u32> for Counter {
    fn register(&mut self, value: u32) {
        self.count += value;
    }

    fn get(&self) -> u32 {
        self.count
    }
}

#[derive(Default, Clone, Copy)]
pub struct MeanTime {
    total: Duration,
    len: usize,
}

impl MetricType<Duration> for MeanTime {
    fn register(&mut self, value: Duration) {
        self.total += value;
        self.len += 1;
    }

    fn get(&self) -> Duration {
        if self.len == 0 {
            Duration::default()
        } else {
            self.total / self.len as u32
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct Time {
    total: Duration,
}

impl MetricType<Duration> for Time {
    fn register(&mut self, value: Duration) {
        self.total += value;
    }

    fn get(&self) -> Duration {
        self.total
    }
}
