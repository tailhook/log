// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A lightweight logging facade.
//!
//! The `log` crate provides a single logging API that abstracts over the
//! actual logging implementation. Libraries can use the logging API provided
//! by this crate, and the consumer of those libraries can choose the logging
//! implementation that is most suitable for its use case.
//!
//! If no logging implementation is selected, the facade falls back to a "noop"
//! implementation that ignores all log messages. The overhead in this case
//! is very small - just an integer load, comparison and jump.
//!
//! A log request consists of a _target_, a _level_, and a _body_. A target is a
//! string which defaults to the module path of the location of the log request,
//! though that default may be overridden. Logger implementations typically use
//! the target to filter requests based on some user configuration.
//!
//! # Use
//!
//! The basic use of the log crate is through the five logging macros: [`error!`],
//! [`warn!`], [`info!`], [`debug!`] and [`trace!`]
//! where `error!` represents the highest-priority log level, and `trace!` the lowest.
//!
//! Each of these macros accept format strings similarly to [`println!`].
//!
//!
//! [`error!`]: ./macro.error.html
//! [`warn!`]: ./macro.warn.html
//! [`info!`]: ./macro.info.html
//! [`debug!`]: ./macro.debug.html
//! [`trace!`]: ./macro.trace.html
//! [`println!`]: https://doc.rust-lang.org/stable/std/macro.println.html
//!
//! ## In libraries
//!
//! Libraries should link only to the `log` crate, and use the provided
//! macros to log whatever information will be useful to downstream consumers.
//!
//! ### Examples
//!
//! ```rust
//! # #![allow(unstable)]
//! #[macro_use]
//! extern crate log;
//!
//! # #[derive(Debug)] pub struct Yak(String);
//! # impl Yak { fn shave(&self, _: u32) {} }
//! # fn find_a_razor() -> Result<u32, u32> { Ok(1) }
//! pub fn shave_the_yak(yak: &Yak) {
//!     info!(target: "yak_events", "Commencing yak shaving for {:?}", yak);
//!
//!     loop {
//!         match find_a_razor() {
//!             Ok(razor) => {
//!                 info!("Razor located: {}", razor);
//!                 yak.shave(razor);
//!                 break;
//!             }
//!             Err(err) => {
//!                 warn!("Unable to locate a razor: {}, retrying", err);
//!             }
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ## In executables
//!
//! Executables should choose a logging implementation and initialize it early in the
//! runtime of the program. Logging implementations will typically include a
//! function to do this. Any log messages generated before
//! the implementation is initialized will be ignored.
//!
//! The executable itself may use the `log` crate to log as well.
//!
//! ### Warning
//!
//! The logging system may only be initialized once.
//!
//! ### Examples
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate log;
//! extern crate env_logger;
//!
//! fn main() {
//!     // Select env_logger, one possible logger implementation
//!     // (see https://doc.rust-lang.org/log/env_logger/index.html)
//!     env_logger::init();
//!
//!     info!("starting up");
//!     error!("error: {}", 404);
//!
//!     // ...
//! }
//! ```
//!
//! # Available logging implementations
//!
//! In order to produce log output executables have to use
//! a logger implementation compatible with the facade.
//! There are many available implementations to chose from,
//! here are some of the most popular ones:
//!
//! * Simple minimal loggers:
//!     * [env_logger]
//!     * [simple_logger]
//!     * [simplelog]
//!     * [pretty_env_logger]
//!     * [stderrlog]
//!     * [flexi_logger]
//! * Complex configurable frameworks:
//!     * [log4rs]
//!     * [fern]
//! * Adaptors for other facilities:
//!     * [syslog]
//!     * [slog-stdlog]
//!
//! # Implementing a Logger
//!
//! Loggers implement the [`Log`] trait. Here's a very basic example that simply
//! logs all messages at the [`Error`][level_link], [`Warn`][level_link] or
//! [`Info`][level_link] levels to stdout:
//!
//! ```rust
//! extern crate log;
//!
//! use log::{Record, Level, Metadata};
//!
//! struct SimpleLogger;
//!
//! impl log::Log for SimpleLogger {
//!     fn enabled(&self, metadata: &Metadata) -> bool {
//!         metadata.level() <= Level::Info
//!     }
//!
//!     fn log(&self, record: &Record) {
//!         if self.enabled(record.metadata()) {
//!             println!("{} - {}", record.level(), record.args());
//!         }
//!     }
//!     fn flush(&self) {}
//! }
//!
//! # fn main() {}
//! ```
//!
//! Loggers are installed by calling the [`set_logger`] function. It takes a
//! closure which is provided a [`MaxLevelFilter`] token and returns a
//! [`Log`] trait object. The [`MaxLevelFilter`] token controls the global
//! maximum log level. The logging facade uses this as an optimization to
//! improve performance of log messages at levels that are disabled. In the
//! case of our example logger, we'll want to set the maximum log level to
//! [`Info`][level_link], since we ignore any [`Debug`][level_link] or
//! [`Trace`][level_link] level log messages. A logging implementation
//! should provide a function that wraps a call to [`set_logger`], handling
//! initialization of the logger:
//!
//! ```rust
//! # extern crate log;
//! # use log::{Level, LevelFilter, SetLoggerError, Metadata};
//! # struct SimpleLogger;
//! # impl log::Log for SimpleLogger {
//! #   fn enabled(&self, _: &Metadata) -> bool { false }
//! #   fn log(&self, _: &log::Record) {}
//! #   fn flush(&self) {}
//! # }
//! # fn main() {}
//! # #[cfg(feature = "use_std")]
//! pub fn init() {
//!     let filter = LevelFilter::Info;
//!     let logger = Box::new(SimpleLogger);
//!     log::set_logger(logger, filter);
//! }
//! ```
//!
//! # Use with `no_std`
//!
//! To use the `log` crate without depending on `libstd`, you need to specify
//! `default-features = false` when specifying the dependency in `Cargo.toml`.
//! This makes no difference to libraries using `log` since the logging API
//! remains the same. However executables will need to use the [`set_logger_raw`]
//! function to initialize a logger.
//!
//! ```rust
//! # extern crate log;
//! # use log::{Level, LevelFilter, Log, SetLoggerError, Metadata};
//! # struct SimpleLogger;
//! # impl log::Log for SimpleLogger {
//! #   fn enabled(&self, _: &Metadata) -> bool { false }
//! #   fn log(&self, _: &log::Record) {}
//! #   fn flush(&self) {}
//! # }
//! # fn main() {}
//! pub fn try_init() -> Result<(), SetLoggerError> {
//!     unsafe {
//!         log::set_logger_raw(|max_level| {
//!             static LOGGER: SimpleLogger = SimpleLogger;
//!             max_level.set(LevelFilter::Info);
//!             &SimpleLogger
//!         })
//!     }
//! }
//! ```
//!
//! # Features
//!
//! Optionally, when defining a `Cargo.toml` file, additional parameters can be passed that affect
//! the logger depending on the target of the build.  Effectively, `max_level_*` and
//! `release_max_level_*` directives can be added as features of the log dependency.  When
//! these are set, they override the behavior of the logging levels above the declared maximum
//! preventing anything higher from logging.
//!
//! ```toml
//! [dependencies.log]
//! version = "^0.3.7"
//! features = ["max_level_debug", "release_max_level_warn"]
//! ```
//!
//! [`Log`]: trait.Log.html
//! [level_link]: enum.Level.html
//! [`set_logger`]: fn.set_logger.html
//! [`MaxLevelFilter`]: struct.MaxLevelFilter.html
//! [`set_logger_raw`]: fn.set_logger_raw.html
//! [env_logger]: https://docs.rs/env_logger/*/env_logger/
//! [simple_logger]: https://github.com/borntyping/rust-simple_logger
//! [simplelog]: https://github.com/drakulix/simplelog.rs
//! [pretty_env_logger]: https://docs.rs/pretty_env_logger/*/pretty_env_logger/
//! [stderrlog]: https://docs.rs/stderrlog/*/stderrlog/
//! [flexi_logger]: https://docs.rs/flexi_logger/*/flexi_logger/
//! [syslog]: https://docs.rs/syslog/*/syslog/
//! [slog-stdlog]: https://docs.rs/slog-stdlog/*/slog_stdlog/
//! [log4rs]: https://docs.rs/log4rs/*/log4rs/
//! [fern]: https://docs.rs/fern/*/fern/

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "https://www.rust-lang.org/favicon.ico",
       html_root_url = "https://docs.rs/log/0.3.8")]
#![warn(missing_docs)]
#![deny(missing_debug_implementations)]

#![cfg_attr(not(feature = "use_std"), no_std)]

// When compiled for the rustc compiler itself we want to make sure that this is
// an unstable crate
#![cfg_attr(rustbuild, feature(staged_api, rustc_private))]
#![cfg_attr(rustbuild, unstable(feature = "rustc_private", issue = "27812"))]

#[cfg(not(feature = "use_std"))]
extern crate core as std;

#[macro_use]
extern crate cfg_if;

use std::cmp;
#[cfg(feature = "use_std")]
use std::error;
use std::fmt;
use std::mem;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

#[macro_use]
mod macros;
mod serde;

// The 0.3.x shim uses this to figure out if it can assume that the file and module_path strings
// are 'static
//
// If you use this, I will hunt you down. >:(
#[doc(hidden)]
pub const __SUPER_SECRET_STRINGS_ARE_STATIC: bool = true;

// There are three different states that we care about: the logger's
// uninitialized, the logger's initializing (set_logger's been called but
// LOGGER hasn't actually been set yet), or the logger's active.
//
// The LOGGER static holds a pointer to the global logger. It is protected by
// the STATE static which determines whether LOGGER has been initialized yet.

static mut LOGGER: *const Log = &NopLogger;
static STATE: AtomicUsize = ATOMIC_USIZE_INIT;

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

static MAX_LOG_LEVEL_FILTER: AtomicUsize = ATOMIC_USIZE_INIT;

static LOG_LEVEL_NAMES: [&'static str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

static SET_LOGGER_ERROR: &'static str = "attempted to set a logger after the logging system \
                     was already initialized";
static LEVEL_PARSE_ERROR: &'static str = "attempted to convert a string that doesn't match an existing log level";

/// An enum representing the available verbosity levels of the logger.
///
/// Typical usage includes: checking if a certain `Level` is enabled with
/// [`log_enabled!`](macro.log_enabled.html), specifying the `Level` of
/// [`log!`](macro.log.html), and comparing a `Level` directly to a
/// [`LevelFilter`](enum.LevelFilter.html).
#[repr(usize)]
#[derive(Copy, Eq, Debug, Hash)]
pub enum Level {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 1, // This way these line up with the discriminants for LevelFilter below
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}

impl Clone for Level {
    #[inline]
    fn clone(&self) -> Level {
        *self
    }
}

impl PartialEq for Level {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialOrd for Level {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}

impl Ord for Level {
    #[inline]
    fn cmp(&self, other: &Level) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

fn ok_or<T, E>(t: Option<T>, e: E) -> Result<T, E> {
    match t {
        Some(t) => Ok(t),
        None => Err(e),
    }
}

// Reimplemented here because std::ascii is not available in libcore
fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    fn to_ascii_uppercase(c: u8) -> u8 {
        if c >= b'a' && c <= b'z' {
            c - b'a' + b'A'
        } else {
            c
        }
    }

    if a.len() == b.len() {
        a.bytes()
            .zip(b.bytes())
            .all(|(a, b)| to_ascii_uppercase(a) == to_ascii_uppercase(b))
    } else {
        false
    }
}

impl FromStr for Level {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<Level, Self::Err> {
        ok_or(LOG_LEVEL_NAMES
                  .iter()
                  .position(|&name| eq_ignore_ascii_case(name, level))
                  .into_iter()
                  .filter(|&idx| idx != 0)
                  .map(|idx| Level::from_usize(idx).unwrap())
                  .next(),
              ParseLevelError(()))
    }
}

impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(LOG_LEVEL_NAMES[*self as usize])
    }
}

impl Level {
    fn from_usize(u: usize) -> Option<Level> {
        match u {
            1 => Some(Level::Error),
            2 => Some(Level::Warn),
            3 => Some(Level::Info),
            4 => Some(Level::Debug),
            5 => Some(Level::Trace),
            _ => None,
        }
    }

    /// Returns the most verbose logging level.
    #[inline]
    pub fn max() -> Level {
        Level::Trace
    }

    /// Converts the `Level` to the equivalent `LevelFilter`.
    #[inline]
    pub fn to_level_filter(&self) -> LevelFilter {
        LevelFilter::from_usize(*self as usize).unwrap()
    }
}

/// An enum representing the available verbosity level filters of the logger.
///
/// A `LevelFilter` may be compared directly to a [`Level`](enum.Level.html).
/// Use this type to [`get()`](struct.MaxLevelFilter.html#method.get) and
/// [`set()`](struct.MaxLevelFilter.html#method.set) the
/// [`MaxLevelFilter`](struct.MaxLevelFilter.html), or to match with the getter
/// [`max_level()`](fn.max_level.html).
#[repr(usize)]
#[derive(Copy, Eq, Debug, Hash)]
pub enum LevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

// Deriving generates terrible impls of these traits

impl Clone for LevelFilter {
    #[inline]
    fn clone(&self) -> LevelFilter {
        *self
    }
}

impl PartialEq for LevelFilter {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        other.eq(self)
    }
}

impl PartialOrd for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<Level> for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        other.partial_cmp(self).map(|x| x.reverse())
    }
}

impl Ord for LevelFilter {
    #[inline]
    fn cmp(&self, other: &LevelFilter) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

impl FromStr for LevelFilter {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<LevelFilter, Self::Err> {
        ok_or(LOG_LEVEL_NAMES
                  .iter()
                  .position(|&name| eq_ignore_ascii_case(name, level))
                  .map(|p| LevelFilter::from_usize(p).unwrap()),
              ParseLevelError(()))
    }
}

impl fmt::Display for LevelFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", LOG_LEVEL_NAMES[*self as usize])
    }
}

impl LevelFilter {
    fn from_usize(u: usize) -> Option<LevelFilter> {
        match u {
            0 => Some(LevelFilter::Off),
            1 => Some(LevelFilter::Error),
            2 => Some(LevelFilter::Warn),
            3 => Some(LevelFilter::Info),
            4 => Some(LevelFilter::Debug),
            5 => Some(LevelFilter::Trace),
            _ => None,
        }
    }
    /// Returns the most verbose logging level filter.
    #[inline]
    pub fn max() -> LevelFilter {
        LevelFilter::Trace
    }

    /// Converts `self` to the equivalent `Level`.
    ///
    /// Returns `None` if `self` is `LevelFilter::Off`.
    #[inline]
    pub fn to_level(&self) -> Option<Level> {
        Level::from_usize(*self as usize)
    }
}

/// The "payload" of a log message.
///
/// # Use
///
/// `Record` structures are passed as parameters to the [`log`][method.log]
/// method of the [`Log`] trait. Logger implementors manipulate these
/// structures in order to display log messages. `Record`s are automatically
/// created by the [`log!`] macro and so are not seen by log users.
///
/// Note that the [`level()`] and [`target()`] accessors are equivalent to
/// `self.metadata().level()` and `self.metadata().target()` respectively.
/// These methods are provided as a convenience for users of this structure.
///
/// # Example
///
/// The following example shows a simple logger that displays the level,
/// module path, and message of any `Record` that is passed to it.
///
/// ```rust
/// # extern crate log;
/// struct SimpleLogger;
///
/// impl log::Log for SimpleLogger {
///    fn enabled(&self, metadata: &log::Metadata) -> bool {
///        true
///    }
///
///    fn log(&self, record: &log::Record) {
///        if !self.enabled(record.metadata()) {
///            return;
///        }
///
///        println!("{}:{} -- {}",
///                 record.level(),
///                 record.module_path(),
///                 record.args());
///    }
///    fn flush(&self) {}
/// }
/// ```
///
/// [method.log]: trait.Log.html#method.log
/// [`Log`]: trait.Log.html
/// [`log!`]: macro.log.html
/// [`level()`]: struct.Record.html#method.level
/// [`target()`]: struct.Record.html#method.target
#[derive(Clone, Debug)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    args: fmt::Arguments<'a>,
    module_path: &'static str,
    file: &'static str,
    line: u32,
}

impl<'a> Record<'a> {
    /// Returns a new builder
    #[inline]
    pub fn builder() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }

    /// The message body.
    #[inline]
    pub fn args(&self) -> &fmt::Arguments<'a> {
        &self.args
    }

    /// Metadata about the log directive.
    #[inline]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.metadata.level()
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &str {
        self.metadata.target()
    }

    /// The module path of the message.
    #[inline]
    pub fn module_path(&self) -> &str {
        self.module_path
    }

    /// The source file containing the message.
    #[inline]
    pub fn file(&self) -> &str {
        self.file
    }

    /// The line containing the message.
    #[inline]
    pub fn line(&self) -> u32 {
        self.line
    }
}

/// Builder for [`Record`](struct.Record.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `RecordBuilder` can set the different parameters of `Record` object, and returns
/// the created object when `build` is called.
///
/// # Examples
///
///
/// ```rust
/// use log::{Level, Record};
///
/// let record = Record::builder()
///                 .args(format_args!("Error!"))
///                 .level(Level::Error)
///                 .target("myApp")
///                 .file("server.rs")
///                 .line(144)
///                 .build();
/// ```
///
/// Alternatively, use [`MetadataBuilder`](struct.Metadatabuilder.html):
///
/// ```rust
/// use log::{Record, Level, MetadataBuilder};
///
/// let error_metadata = MetadataBuilder::new()
///                         .target("myApp")
///                         .level(Level::Error)
///                         .build();
///
/// let record = Record::builder()
///                 .metadata(error_metadata)
///                 .args(format_args!("Error!"))
///                 .line(433)
///                 .file("app.rs")
///                 .module_path("server/")
///                 .build();
/// ```

#[derive(Debug)]
pub struct RecordBuilder<'a> {
    record: Record<'a>
}

impl<'a> RecordBuilder<'a> {
    /// Construct new `RecordBuilder`.
    ///
    /// The default options are:
    ///
    /// - `args`: [`format_args!("")`]
    /// - `metadata`: [`Metadata::builder().build()`]
    /// - `module_path`: `""`
    /// - `file`: `""`
    /// - `line`: `0`
    /// [`format_args!("")`]: https://doc.rust-lang.org/std/macro.format_args.html
    /// [`Metadata::builder().build()`]: struct.MetadataBuilder.html#method.build
    #[inline]
    pub fn new() -> RecordBuilder<'a> {
        RecordBuilder {
            record: Record {
                args: format_args!(""),
                metadata: Metadata::builder().build(),
                module_path: "",
                file: "",
                line: 0,
            }
        }
    }

    /// Set [`args`](struct.Record.html#method.args).
    #[inline]
    pub fn args(&mut self, args: fmt::Arguments<'a>) -> &mut RecordBuilder<'a> {
        self.record.args = args;
        self
    }

    /// Set [`metadata`](struct.Record.html#method.metadata). Construct a `Metadata` object with [`MetadataBuilder`](struct.MetadataBuilder.html).
    #[inline]
    pub fn metadata(&mut self, metadata: Metadata<'a>) -> &mut RecordBuilder<'a> {
        self.record.metadata = metadata;
        self
    }

    /// Set [`Metadata::level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut RecordBuilder<'a> {
        self.record.metadata.level = level;
        self
    }

    /// Set [`Metadata::target`](struct.Metadata.html#method.target)
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut RecordBuilder<'a> {
        self.record.metadata.target = target;
        self
    }

    /// Set [`module_path`](struct.Record.html#method.module_path)
    #[inline]
    pub fn module_path(&mut self, path: &'static str) -> &mut RecordBuilder<'a> {
        self.record.module_path = path;
        self
    }

    /// Set [`file`](struct.Record.html#method.file)
    #[inline]
    pub fn file(&mut self, file: &'static str) -> &mut RecordBuilder<'a> {
        self.record.file = file;
        self
    }

    /// Set [`line`](struct.Record.html#method.line)
    #[inline]
    pub fn line(&mut self, line: u32) -> &mut RecordBuilder<'a> {
        self.record.line = line;
        self
    }

    /// Invoke the builder and return a `Record`
    #[inline]
    pub fn build(&self) -> Record<'a> {
        self.record.clone()
    }
}

/// Metadata about a log message.
///
/// # Use
///
/// `Metadata` structs are created when users of the library use
/// logging macros.
///
/// They are consumed by implementations of the `Log` trait in the
/// `enabled` method.
///
/// `Record`s use `Metadata` to determine the log message's severity
/// and target.
///
/// Users should use the `log_enabled!` macro in their code to avoid
/// constructing expensive log messages.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate log;
/// #
/// use log::{Record, Level, Metadata};
///
/// struct MyLogger;
///
/// impl log::Log for MyLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("{} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// # fn main(){}
/// ```
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}

impl<'a> Metadata<'a> {
    /// Returns a new builder
    #[inline]
    pub fn builder() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &str {
        self.target
    }
}

/// Builder for [`Metadata`](struct.Metadata.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `MetadataBuilder` can set the different parameters of a `Metadata` object, and returns
/// the created object when `build` is called.
///
/// # Example
///
/// ```rust
/// let target = "myApp";
/// use log::{Level, MetadataBuilder};
/// let metadata = MetadataBuilder::new()
///                     .level(Level::Debug)
///                     .target(target)
///                     .build();
/// ```
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct MetadataBuilder<'a> {
    metadata: Metadata<'a>
}

impl<'a> MetadataBuilder<'a> {
    /// Construct a new `MetadataBuilder`.
    ///
    /// The default options are:
    ///
    /// - `level`: `Level::Info`
    /// - `target`: `""`
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder {
            metadata: Metadata {
                level: Level::Info,
                target: "",
            }
        }
    }

    /// Setter for [`level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, arg: Level) -> &mut MetadataBuilder<'a> {
        self.metadata.level = arg;
        self
    }

    /// Setter for [`target`](struct.Metadata.html#method.target).
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut MetadataBuilder<'a> {
        self.metadata.target = target;
        self
    }

    /// Returns a `Metadata` object.
    #[inline]
    pub fn build(&self) -> Metadata<'a> {
        self.metadata.clone()
    }
}

/// A trait encapsulating the operations required of a logger
pub trait Log: Sync + Send {
    /// Determines if a log message with the specified metadata would be
    /// logged.
    ///
    /// This is used by the `log_enabled!` macro to allow callers to avoid
    /// expensive computation of log message arguments if the message would be
    /// discarded anyway.
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Logs the `Record`.
    ///
    /// Note that `enabled` is *not* necessarily called before this method.
    /// Implementations of `log` should perform all necessary filtering
    /// internally.
    fn log(&self, record: &Record);

    /// Flushes any buffered records.
    fn flush(&self);
}

// Just used as a dummy initial value for LOGGER
struct NopLogger;

impl Log for NopLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        false
    }

    fn log(&self, _: &Record) {}
    fn flush(&self) {}
}

/// A token providing read and write access to the global maximum log level
/// filter.
///
/// The maximum log level is used as an optimization to avoid evaluating log
/// messages that will be ignored by the logger. Any message with a level
/// higher than the maximum log level filter will be ignored. A logger should
/// make sure to keep the maximum log level filter in sync with its current
/// configuration.
#[allow(missing_copy_implementations)]
pub struct MaxLevelFilter(());

impl fmt::Debug for MaxLevelFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "MaxLevelFilter")
    }
}

impl MaxLevelFilter {
    /// Gets the current maximum log level filter.
    pub fn get(&self) -> LevelFilter {
        max_level()
    }

    /// Sets the maximum log level.
    pub fn set(&self, level: LevelFilter) {
        MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::SeqCst)
    }
}

/// Returns the current maximum log level.
///
/// The [`log!`], [`error!`], [`warn!`], [`info!`], [`debug!`], and [`trace!`] macros check
/// this value and discard any message logged at a higher level. The maximum
/// log level is set by the `MaxLevel` token passed to loggers.
///
/// [`log!`]: macro.log.html
/// [`error!`]: macro.error.html
/// [`warn!`]: macro.warn.html
/// [`info!`]: macro.info.html
/// [`debug!`]: macro.debug.html
/// [`trace!`]: macro.trace.html
#[inline(always)]
pub fn max_level() -> LevelFilter {
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}

/// Sets the global logger.
///
/// The `make_logger` closure is passed a `MaxLevel` object, which the
/// logger should use to keep the global maximum log level in sync with the
/// highest log level that the logger will not ignore.
///
/// This function may only be called once in the lifetime of a program. Any log
/// events that occur before the call to `try_set_logger` completes will be
/// ignored.
///
/// This function does not typically need to be called manually. Logger
/// implementations should provide an initialization method that calls
/// `try_set_logger` internally.
///
/// Requires the `use_std` feature (enabled by default).
///
/// # Errors
///
/// This function fails to set the global logger if it has already
/// been called before.
///
/// # Example
///
/// Implements a custom logger `ConsoleLogger` which prints to stdout.
/// In order to use the logging macros, `ConsoleLogger` implements
/// the [`Log`] trait and has to be installed via `set_logger`.
///
/// ```rust
/// # #[macro_use]
/// # extern crate log;
/// #
/// use log::{Record, Level, Metadata, LevelFilter, SetLoggerError};
///
/// struct ConsoleLogger;
///
/// impl log::Log for ConsoleLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("Rust says: {} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// fn try_init() -> Result<(), SetLoggerError> {
///     log::try_set_logger(|max_log_level| {
///                         max_log_level.set(LevelFilter::Info);
///                         Box::new(ConsoleLogger)
///                     });
///
///     info!("hello log");
///     warn!("warning");
///     error!("oops");
///     Ok(())
/// }
/// #
/// # fn main(){}
/// ```
///
/// [`Log`]: trait.Log.html
#[cfg(feature = "use_std")]
pub fn try_set_logger<M>(make_logger: M) -> Result<(), SetLoggerError>
    where M: FnOnce(MaxLevelFilter) -> Box<Log>
{
    unsafe { set_logger_raw(|max_level| mem::transmute(make_logger(max_level))) }
}

/// Sets the global logger.
///
/// This function may only be called once in the lifetime of a program. Any log
/// events that occur before the call to `set_logger` completes will be
/// ignored.
///
/// This function will panic on future initialization attempts.
///
/// This function does not typically need to be called manually. Logger
/// implementations should provide an initialization method that calls
/// `set_logger` internally.
///
/// Requires the `use_std` feature (enabled by default).
///
/// # Panics
///
/// The function will panic if it is called more than once.
#[cfg(feature = "use_std")]
pub fn set_logger(logger: Box<Log>, filter: LevelFilter) {
    match try_set_logger(|max| { max.set(filter); logger }) {
        Ok(()) => {}
        Err(_) => panic!("global logger is already initialized"),
    }
}

/// Sets the global logger from a raw pointer.
///
/// This function is similar to [`set_logger`] except that it is usable in
/// `no_std` code.
///
/// The `make_logger` closure is passed a `MaxLevel` object, which the
/// logger should use to keep the global maximum log level in sync with the
/// highest log level that the logger will not ignore.
///
/// This function may only be called once in the lifetime of a program. Any log
/// events that occur before the call to `set_logger_raw` completes will be
/// ignored.
///
/// This function does not typically need to be called manually. Logger
/// implementations should provide an initialization method that calls
/// `set_logger_raw` internally.
///
/// # Errors
///
/// This function fails to set the global logger if [`set_logger`]
/// has already been called.
///
/// # Safety
///
/// The pointer returned by `make_logger` must remain valid for the entire
/// duration of the program.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate log;
/// #
/// use log::{Record, Level, Metadata, LevelFilter};
///
/// struct MyLogger;
///
/// const MY_LOGGER: MyLogger = MyLogger;
///
/// impl log::Log for MyLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("{} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// # fn main(){
/// unsafe {
///     log::set_logger_raw(|max_log_level| {
///                         max_log_level.set(LevelFilter::Info);
///                         &MY_LOGGER as *const MyLogger
///                         })
/// };
///
///    info!("hello log");
///    warn!("warning");
///    error!("oops");
/// # }
/// ```
///
/// [`set_logger`]: fn.set_logger.html
pub unsafe fn set_logger_raw<M>(make_logger: M) -> Result<(), SetLoggerError>
    where M: FnOnce(MaxLevelFilter) -> *const Log
{
    if STATE.compare_and_swap(UNINITIALIZED, INITIALIZING, Ordering::SeqCst) != UNINITIALIZED {
        return Err(SetLoggerError(()));
    }

    LOGGER = make_logger(MaxLevelFilter(()));
    STATE.store(INITIALIZED, Ordering::SeqCst);
    Ok(())
}

/// The type returned by [`set_logger`] if [`set_logger`] has already been called.
/// [`set_logger`]: fn.set_logger.html
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct SetLoggerError(());

impl fmt::Display for SetLoggerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(SET_LOGGER_ERROR)
    }
}

// The Error trait is not available in libcore
#[cfg(feature = "use_std")]
impl error::Error for SetLoggerError {
    fn description(&self) -> &str {
        SET_LOGGER_ERROR
    }
}

/// The type returned by [`from_str`] when the string doesn't match any of the log levels.
/// [`from_str`]: https://doc.rust-lang.org/std/str/trait.FromStr.html#tymethod.from_str
#[allow(missing_copy_implementations)]
#[derive(Debug, PartialEq)]
pub struct ParseLevelError(());

impl fmt::Display for ParseLevelError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(LEVEL_PARSE_ERROR)
    }
}

// The Error trait is not available in libcore
#[cfg(feature = "use_std")]
impl error::Error for ParseLevelError {
    fn description(&self) -> &str {
        LEVEL_PARSE_ERROR
    }
}

/// Returns a reference to the logger.
///
/// If a logger has not been set, a no-op implementation is returned.
pub fn logger() -> &'static Log {
    unsafe {
        if STATE.load(Ordering::SeqCst) != INITIALIZED {
            static NOP: NopLogger = NopLogger;
            &NOP
        } else {
            &*LOGGER
        }
    }
}

/// The statically resolved maximum log level.
///
/// See the crate level documentation for information on how to configure this.
///
/// This value is checked by the log macros, but not by the `log` and `enabled`
/// functions. Code that manually calls those functions should compare the level
/// against this value.
pub const STATIC_MAX_LEVEL: LevelFilter = MAX_LEVEL_INNER;

cfg_if! {
    if #[cfg(all(not(debug_assertions), feature = "release_max_level_off"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Off;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_error"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Error;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_warn"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Warn;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_info"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Info;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_debug"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Debug;
    } else if #[cfg(all(not(debug_assertions), feature = "release_max_level_trace"))] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Trace;
    } else if #[cfg(feature = "max_level_off")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Off;
    } else if #[cfg(feature = "max_level_error")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Error;
    } else if #[cfg(feature = "max_level_warn")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Warn;
    } else if #[cfg(feature = "max_level_info")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Info;
    } else if #[cfg(feature = "max_level_debug")] {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Debug;
    } else {
        const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Trace;
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use tests::std::string::ToString;
    use super::{Level, LevelFilter, ParseLevelError};

    #[test]
    fn test_levelfilter_from_str() {
        let tests = [("off", Ok(LevelFilter::Off)),
                     ("error", Ok(LevelFilter::Error)),
                     ("warn", Ok(LevelFilter::Warn)),
                     ("info", Ok(LevelFilter::Info)),
                     ("debug", Ok(LevelFilter::Debug)),
                     ("trace", Ok(LevelFilter::Trace)),
                     ("OFF", Ok(LevelFilter::Off)),
                     ("ERROR", Ok(LevelFilter::Error)),
                     ("WARN", Ok(LevelFilter::Warn)),
                     ("INFO", Ok(LevelFilter::Info)),
                     ("DEBUG", Ok(LevelFilter::Debug)),
                     ("TRACE", Ok(LevelFilter::Trace)),
                     ("asdf", Err(ParseLevelError(())))];
        for &(s, ref expected) in &tests {
            assert_eq!(expected, &s.parse());
        }
    }

    #[test]
    fn test_level_from_str() {
        let tests = [("OFF", Err(ParseLevelError(()))),
                     ("error", Ok(Level::Error)),
                     ("warn", Ok(Level::Warn)),
                     ("info", Ok(Level::Info)),
                     ("debug", Ok(Level::Debug)),
                     ("trace", Ok(Level::Trace)),
                     ("ERROR", Ok(Level::Error)),
                     ("WARN", Ok(Level::Warn)),
                     ("INFO", Ok(Level::Info)),
                     ("DEBUG", Ok(Level::Debug)),
                     ("TRACE", Ok(Level::Trace)),
                     ("asdf", Err(ParseLevelError(())))];
        for &(s, ref expected) in &tests {
            assert_eq!(expected, &s.parse());
        }
    }

    #[test]
    fn test_level_show() {
        assert_eq!("INFO", Level::Info.to_string());
        assert_eq!("ERROR", Level::Error.to_string());
    }

    #[test]
    fn test_levelfilter_show() {
        assert_eq!("OFF", LevelFilter::Off.to_string());
        assert_eq!("ERROR", LevelFilter::Error.to_string());
    }

    #[test]
    fn test_cross_cmp() {
        assert!(Level::Debug > LevelFilter::Error);
        assert!(LevelFilter::Warn < Level::Trace);
        assert!(LevelFilter::Off < Level::Error);
    }

    #[test]
    fn test_cross_eq() {
        assert!(Level::Error == LevelFilter::Error);
        assert!(LevelFilter::Off != Level::Error);
        assert!(Level::Trace == LevelFilter::Trace);
    }

    #[test]
    fn test_to_level() {
        assert_eq!(Some(Level::Error), LevelFilter::Error.to_level());
        assert_eq!(None, LevelFilter::Off.to_level());
        assert_eq!(Some(Level::Debug), LevelFilter::Debug.to_level());
    }

    #[test]
    fn test_to_level_filter() {
        assert_eq!(LevelFilter::Error, Level::Error.to_level_filter());
        assert_eq!(LevelFilter::Trace, Level::Trace.to_level_filter());
    }

    #[test]
    #[cfg(feature = "use_std")]
    fn test_error_trait() {
        use std::error::Error;
        use super::SetLoggerError;
        let e = SetLoggerError(());
        assert_eq!(e.description(),
                   "attempted to set a logger after the logging system \
                     was already initialized");
    }

    #[test]
    fn test_metadata_builder() {
        use super::MetadataBuilder;
        let target = "myApp";
        let metadata_test = MetadataBuilder::new()
                            .level(Level::Debug)
                            .target(target)
                            .build();
        assert_eq!(metadata_test.level(), Level::Debug);
        assert_eq!(metadata_test.target(), "myApp");
    }

    #[test]
    fn test_metadata_convenience_builder() {
        use super::Metadata;
        let target = "myApp";
        let metadata_test = Metadata::builder()
                            .level(Level::Debug)
                            .target(target)
                            .build();
        assert_eq!(metadata_test.level(), Level::Debug);
        assert_eq!(metadata_test.target(), "myApp");
    }

    #[test]
    fn test_record_builder() {
        use super::{MetadataBuilder, RecordBuilder};
        let target = "myApp";
        let metadata = MetadataBuilder::new()
                        .target(target)
                        .build();
        let fmt_args = format_args!("hello");
        let record_test = RecordBuilder::new()
                            .args(fmt_args)
                            .metadata(metadata)
                            .module_path("foo")
                            .file("bar")
                            .line(30)
                            .build();
        assert_eq!(record_test.metadata().target(), "myApp");
        assert_eq!(record_test.module_path(), "foo");
        assert_eq!(record_test.file(), "bar");
        assert_eq!(record_test.line(), 30);
    }

    #[test]
    fn test_record_convenience_builder() {
        use super::{Metadata, Record};
        let target = "myApp";
        let metadata = Metadata::builder()
                        .target(target)
                        .build();
        let fmt_args = format_args!("hello");
        let record_test = Record::builder()
                            .args(fmt_args)
                            .metadata(metadata)
                            .module_path("foo")
                            .file("bar")
                            .line(30)
                            .build();
        assert_eq!(record_test.target(), "myApp");
        assert_eq!(record_test.module_path(), "foo");
        assert_eq!(record_test.file(), "bar");
        assert_eq!(record_test.line(), 30);
    }

    #[test]
    fn test_record_complete_builder() {
        use super::{Record, Level};
        let target = "myApp";
        let record_test = Record::builder()
            .module_path("foo")
            .file("bar")
            .line(30)
            .target(target)
            .level(Level::Error)
            .build();
        assert_eq!(record_test.target(), "myApp");
        assert_eq!(record_test.level(), Level::Error);
        assert_eq!(record_test.module_path(), "foo");
        assert_eq!(record_test.file(), "bar");
        assert_eq!(record_test.line(), 30);
    }
}
