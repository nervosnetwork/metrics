#[macro_use]
extern crate criterion;

use criterion::{Benchmark, Criterion};

use metrics::{counter, Key, Recorder, Counter, Gauge, Histogram, CounterHandle, GaugeHandle, HistogramHandle};
use rand::{thread_rng, Rng};

use std::sync::atomic::{AtomicU64, Ordering};

static TEST_HANDLE: TestHandle = TestHandle::new();
static TEST_HANDLE_REF: &'static TestHandle = &TEST_HANDLE;

struct TestHandle(AtomicU64);

impl TestHandle {
    pub const fn new() -> TestHandle {
        TestHandle(AtomicU64::new(0))
    }
}

impl Counter for TestHandle {
    fn increment_counter(&self, value: u64) {
        self.0.fetch_add(value, Ordering::SeqCst);
    }
}

impl Gauge for TestHandle {
    fn update_gauge(&self, _value: f64) {}
}

impl Histogram for TestHandle {
    fn record_histogram(&self, _value: f64) {}
}

#[derive(Default)]
struct TestRecorder;
impl Recorder for TestRecorder {
    fn register_counter(&self, _key: Key, _description: Option<&'static str>) -> CounterHandle {
        TEST_HANDLE_REF.into()
    }
    fn register_gauge(&self, _key: Key, _description: Option<&'static str>) -> GaugeHandle {
        TEST_HANDLE_REF.into()
    }
    fn register_histogram(&self, _key: Key, _description: Option<&'static str>) -> HistogramHandle {
        TEST_HANDLE_REF.into()
    }
}

fn reset_recorder() {
    let recorder = unsafe { &*Box::into_raw(Box::new(TestRecorder::default())) };
    unsafe { metrics::set_recorder_racy(recorder).unwrap() }
}

fn macro_benchmark(c: &mut Criterion) {
    c.bench(
        "macros",
        Benchmark::new("uninitialized/no labels", |b| {
            b.iter(|| {
                counter!("counter_bench", 42);
            });
        })
        .with_function("uninitialized/with static labels", |b| {
            b.iter(|| {
                counter!("counter_bench", 42, "request" => "http", "svc" => "admin");
            });
        })
        .with_function("initialized/no labels", |b| {
            reset_recorder();
            b.iter(|| {
                counter!("counter_bench", 42);
            });
            metrics::clear_recorder();
        })
        .with_function("initialized/with static labels", |b| {
            reset_recorder();
            b.iter(|| {
                counter!("counter_bench", 42, "request" => "http", "svc" => "admin");
            });
            metrics::clear_recorder();
        })
        .with_function("initialized/with dynamic labels", |b| {
            let label_val = thread_rng().gen::<u64>().to_string();

            reset_recorder();
            b.iter(move || {
                counter!("counter_bench", 42, "request" => "http", "uid" => label_val.clone());
            });
            metrics::clear_recorder();
        }),
    );
}

criterion_group!(benches, macro_benchmark);
criterion_main!(benches);
