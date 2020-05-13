//! A lightweight metrics facade.
//!
//! The `metrics` crate provides a single metrics API that abstracts over the actual metrics
//! implementation.  Libraries can use the metrics API provided by this crate, and the consumer of
//! those libraries can choose the metrics implementation that is most suitable for its use case.
//!
//! The `metrics` crate provides a simple but powerful API for collecting metrics in both libraries
//! and binaries.  Library authors and application authors are empowered to liberally instrument
//! their code, paying essentially no cost for doing so until an application opts in.
//!
//! # Use
//! The basic use of the facade crate is through the three metrics macros: [`counter!`], [`gauge!`],
//! and [`histogram!`].  These macros correspond to updating a counter, updating a gauge, and updating
//! a histogram.
//!
//! ## In libraries
//! Libraries should link only to the `metrics` crate, and use the provided macros to record
//! whatever metrics will be useful to downstream consumers.
//!
//! ### Examples
//!
//! ```rust
//! use metrics::{histogram, counter};
//!
//! # use std::time::Instant;
//! # pub fn run_query(_: &str) -> u64 { 42 }
//! pub fn process(query: &str) -> u64 {
//!     let start = Instant::now();
//!     let row_count = run_query(query);
//!     let delta = Instant::now() - start;
//!
//!     histogram!("process.query_time", delta.as_secs_f64());
//!     counter!("process.query_row_count", row_count);
//!
//!     row_count
//! }
//! # fn main() {}
//! ```
//!
//! ## In executables
//!
//! Executables should choose a metrics implementation and initialize it early in the runtime of
//! the program.  Metrics implementations will typically include a function to do this.  Any
//! metrics recordered before the implementation is initialized will be ignored.
//!
//! The executable itself may use the `metrics` crate to record metrics well.
//!
//! ### Warning
//!
//! The metrics system may only be initialized once.
//!
//! # Available metrics implementations
//!
//! Some batteries-included exporters are available to experiment or integrate with metrics systems
//! that you may already use or be familiar with:
//!
//! * [metrics-exporter-tcp]: serve metrics over TCP backed by Protobuf
//! * [metrics-exporter-prometheus]: serve metrics over HTTP in the Prometheus format
//!
//! # Implementing a Recorder
//!
//! At the core, the [`Recorder`] trait is the entry point to an installed recorder, allowing
//! metrics to be registered, as well as returning the necessary handles for metrics to be
//! configured at their respective callsites.
//!
//! Here's an example of a recorder that simply logs each metric update via the `log` crate:
//!
//! ```rust
//! use log::info;
//! use metrics::{Key, Recorder, Counter, Gauge, Histogram, handles::*};
//!
//! struct LogRecorder;
//! struct LoggableMetric(Key);
//!
//! impl LoggableMetric {
//!     pub fn new(key: Key) -> LoggableMetric {
//!         LoggableMetric(key)
//!     }
//! }
//!
//! impl Recorder for LogRecorder {
//!     fn register_counter(&self, key: Key, _description: Option<&'static str>) -> CounterHandle {
//!         CounterHandle::from(LoggableMetric::new(key))
//!     }
//!
//!     fn register_gauge(&self, key: Key, _description: Option<&'static str>) -> GaugeHandle {
//!         GaugeHandle::from(LoggableMetric::new(key))
//!     }
//!
//!     fn register_histogram(&self, key: Key, _description: Option<&'static str>) -> HistogramHandle {
//!         HistogramHandle::from(LoggableMetric::new(key))
//!     }
//! }
//!
//! impl Counter for LoggableMetric
//!     fn increment_counter(&self, value: u64) {
//!         info!("counter '{}' -> {}", self.0, value);
//!     }
//! }
//!
//! impl Gauge for LoggableMetric {
//!     fn update_gauge(&self, value: f64) {
//!         info!("gauge '{}' -> {}", self.0, value);
//!     }
//! }
//!
//! impl Histogram for LoggableMetric {
//!     fn record_histogram(&self, value: f64) {
//!         info!("histogram '{}' -> {}", self.0, value);
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! Recorders are installed by calling the [`set_recorder`] function.  Recorders should provide a
//! function that wraps the creation and installation of the recorder:
//!
//! ```rust
//! # use metrics::{Recorder, Key, SetRecorderError, handles::*};
//! # struct SimpleRecorder;
//! # impl Recorder for SimpleRecorder {
//! #     fn register_counter(&self, _key: Key, _description: Option<&'static str>) -> CounterHandle { CounterHandle::default() }
//! #     fn register_gauge(&self, _key: Key, _description: Option<&'static str>) -> GaugeHandle { GaugeHandle::default() }
//! #     fn register_histogram(&self, _key: Key, _description: Option<&'static str>) -> HistogramHandle { HistogramHandle::default() }
//! # }
//! static RECORDER: SimpleRecorder = SimpleRecorder;
//!
//! pub fn init() -> Result<(), SetRecorderError> {
//!     metrics::set_recorder(&RECORDER)
//! }
//! # fn main() {}
//! ```
//!
//! # Use with `std`
//!
//! `set_recorder` requires you to provide a `&'static Recorder`, which can be hard to
//! obtain if your recorder depends on some runtime configuration.  The `set_boxed_recorder`
//! function is available with the `std` Cargo feature.  It is identical to `set_recorder` except
//! that it takes a `Box<Recorder>` rather than a `&'static Recorder`:
//!
//! ```rust
//! # use metrics::{Recorder, Key, SetRecorderError};
//! # struct SimpleRecorder;
//! # impl Recorder for SimpleRecorder {
//! #     fn register_counter(&self, _key: Key, _description: Option<&'static str>) -> CounterHandle { CounterHandle::default() }
//! #     fn register_gauge(&self, _key: Key, _description: Option<&'static str>) -> GaugeHandle { GaugeHandle::default() }
//! #     fn register_histogram(&self, _key: Key, _description: Option<&'static str>) -> HistogramHandle { HistogramHandle::default() }
//! # }
//!
//! # #[cfg(feature = "std")]
//! pub fn init() -> Result<(), SetRecorderError> {
//!     metrics::set_boxed_recorder(Box::new(SimpleRecorder))
//! }
//! # fn main() {}
//! ```
//!
//!
//!
//! todo: a docs module where individual modules represent different topics? same thing that mio
//! did
//!
//!
//!
//!
//!
//! [metrics-runtime]: https://docs.rs/metrics-runtime
#![deny(missing_docs)]
use proc_macro_hack::proc_macro_hack;

mod common;
pub use self::common::*;

mod key;
pub use self::key::*;

pub mod handles;

mod recorder;
pub use self::recorder::*;

mod macros;
pub use self::macros::*;

/// Registers a counter.
#[proc_macro_hack]
pub use metrics_macros::register_counter;

/// Registers a gauge.
#[proc_macro_hack]
pub use metrics_macros::register_gauge;

/// Registers a histogram.
#[proc_macro_hack]
pub use metrics_macros::register_histogram;

/// Increments a counter.
#[proc_macro_hack]
pub use metrics_macros::increment;

/// Increments a counter.
#[proc_macro_hack]
pub use metrics_macros::counter;

/// Updates a gauge.
#[proc_macro_hack]
pub use metrics_macros::gauge;

/// Records a histogram.
#[proc_macro_hack]
pub use metrics_macros::histogram;
