use std::time::{Duration, Instant};

#[derive(Copy, Clone)]
pub struct MetricsExport<T> {
    pub at: Instant,
    pub duration: Duration,
    pub metrics: T,
}

impl<T: Default> Default for MetricsExport<T> {
    fn default() -> Self {
        Self {
            at: Instant::now(),
            duration: Duration::ZERO,
            metrics: T::default(),
        }
    }
}

impl<T> MetricsExport<T> {
    pub fn new(interval: Duration, metrics: T) -> Self {
        Self {
            at: Instant::now(),
            duration: interval,
            metrics,
        }
    }

    #[allow(dead_code)]
    pub fn secs_ratio(&self) -> f64 {
        1f64 / self.duration.as_secs_f64()
    }
}

pub struct MetricsHandler<T> {
    enabled: bool,
    update_interval: Duration,
    last_update: Instant,

    last_metrics: MetricsExport<T>,
    metrics: T,
}

impl<T: Metrics + Default + Copy> MetricsHandler<T> {
    pub fn new(enabled: bool, update_interval: Duration) -> Self {
        Self {
            enabled,
            update_interval,
            last_update: Instant::now(),
            last_metrics: MetricsExport::new(update_interval, T::default()),
            metrics: T::default(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        if now - self.last_update > self.update_interval {
            self.update_metrics();
            self.last_update = now;
        }
    }

    fn update_metrics(&mut self) {
        self.last_metrics = MetricsExport::new(self.update_interval, self.metrics);
        self.metrics = T::default();
    }

    pub fn export(&self) -> MetricsExport<T> {
        self.last_metrics
    }

    pub fn count(&mut self, field: T::Field, value: u32) {
        if self.enabled {
            self.metrics.count(field, value);
        }
    }

    pub fn mean_time(&mut self, field: T::Field, value: Duration) {
        if self.enabled {
            self.metrics.mean_time(field, value);
        }
    }
}

pub trait Metrics {
    type Field;

    fn count(&mut self, _field: Self::Field, _value: u32) {
        unreachable!()
    }

    fn mean_time(&mut self, _field: Self::Field, _value: Duration) {
        unreachable!()
    }
}
