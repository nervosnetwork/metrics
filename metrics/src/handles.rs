//! Proxies between the macro callsite and recorder.
//!
//! Handles are the proxies that shuttle actual metrics operations -- incrementing a counter or
//! recording a value to a histogram -- to the underlying recorder.
//!
//! Coupled with the callsite caching performed by the macros, handles provide the means to
//! interact with a dynamically-installed recorder with almost no overhead.
use crate::recorder::{Counter, Gauge, Histogram};

static NOOP_COUNTER: &'static dyn Counter = &NoopCounter;
static NOOP_GAUGE: &'static dyn Gauge = &NoopGauge;
static NOOP_HISTOGRAM: &'static dyn Histogram = &NoopHistogram;

/// A counter handle.
pub struct CounterHandle(&'static dyn Counter);

impl CounterHandle {
    /// Increment the counter by the given amount.
    pub fn increment_counter(&self, value: u64) {
        self.0.increment_counter(value);
    }
}

impl Default for CounterHandle {
    fn default() -> Self {
        CounterHandle(NOOP_COUNTER)
    }
}

impl<T> From<&'static T> for CounterHandle
where
    T: Counter
{
    fn from(from: &'static T) -> CounterHandle {
        CounterHandle(from)
    }
}

struct NoopCounter;

impl Counter for NoopCounter {
    fn increment_counter(&self, _value: u64) {}
}

/// A gauge.
pub struct GaugeHandle(&'static dyn Gauge);

impl GaugeHandle {
    /// Set the gauge to the given value.
    pub fn update_gauge(&self, value: f64) {
        self.0.update_gauge(value);
    }
}

impl Default for GaugeHandle {
    fn default() -> Self {
        GaugeHandle(NOOP_GAUGE)
    }
}

impl<T> From<&'static T> for GaugeHandle
where
    T: Gauge
{
    fn from(from: &'static T) -> GaugeHandle {
        GaugeHandle(from)
    }
}

struct NoopGauge;

impl Gauge for NoopGauge {
    fn update_gauge(&self, _value: f64) {}
}

/// A histogram.
pub struct HistogramHandle(&'static dyn Histogram);

impl HistogramHandle {
    /// Records a value to the histogram.
    pub fn record_histogram(&self, value: f64) {
        self.0.record_histogram(value);
    }
}

impl Default for HistogramHandle {
    fn default() -> Self {
        HistogramHandle(NOOP_HISTOGRAM)
    }
}

impl<T> From<&'static T> for HistogramHandle
where
    T: Histogram
{
    fn from(from: &'static T) -> HistogramHandle {
        HistogramHandle(from)
    }
}

struct NoopHistogram;

impl Histogram for NoopHistogram {
    fn record_histogram(&self, _value: f64) {}
}
