use super::metric_type::{Counter, MetricType};
use super::metrics::Metrics;

#[allow(nonstandard_style)]
pub enum PpuMetricFields {
    FRAME_RATE,
}

#[derive(Copy, Clone, Default)]
pub struct PpuMetrics {
    pub frame_rate: Counter,
}

impl Metrics for PpuMetrics {
    type Field = PpuMetricFields;

    fn count(&mut self, field: Self::Field, value: u32) {
        let f = match field {
            PpuMetricFields::FRAME_RATE => &mut self.frame_rate,
        };
        f.register(value);
    }
}
