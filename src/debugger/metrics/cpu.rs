use super::metric_type::{Counter, MeanTime, MetricType};
use super::metrics::Metrics;

#[allow(nonstandard_style)]
pub enum CpuMetricFields {
    INSTRUCTIONS,
    CYCLES,
    TICK_TIME,
}

#[derive(Copy, Clone, Default)]
pub struct CpuMetrics {
    pub instructions: Counter,
    pub cycles: Counter,
    pub tick_time: MeanTime,
}

impl Metrics for CpuMetrics {
    type Field = CpuMetricFields;

    fn count(&mut self, field: Self::Field, value: u32) {
        let f = match field {
            CpuMetricFields::INSTRUCTIONS => &mut self.instructions,
            CpuMetricFields::CYCLES => &mut self.cycles,
            _ => unreachable!(),
        };
        f.register(value);
    }

    fn mean_time(&mut self, field: Self::Field, value: std::time::Duration) {
        let f = match field {
            CpuMetricFields::TICK_TIME => &mut self.tick_time,
            _ => unreachable!(),
        };
        f.register(value);
    }
}
