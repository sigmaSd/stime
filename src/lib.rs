#![warn(missing_docs)]
//!Chrono
//!
//!Easy API to time code.
//!
//!
//!It exposes 2 macros:
//!- [start] => start the timer
//!- [check] => print the elapsed duration since the last start (and the delta between checks)
//!
//!By default these macros are no-op, they are only activated if the environment variable
//!`STIME` is set, example: `STIME=1`
//!
//!There are also some convenience methods under advanced module.
//!
//!
//!```rust,no_run
//!# let expensive_call = || 0;
//!use stime::{start, check};
//!
//!start!("Before evaluating");
//!let eval_result: String = todo!();
//!check!("After evaluating");
//!println!("{}", eval_result);
//!check!("After printing");
//!
//!start!();
//!let _ = expensive_call();
//!check!("After expensive call");
//!```

use once_cell::sync::Lazy;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

#[doc(hidden)]
pub use scolor::*;

#[doc(hidden)]
pub static CHRONO: Lazy<Mutex<Instant>> = Lazy::new(|| Mutex::new(Instant::now()));

#[doc(hidden)]
pub static LAST_DURATION: Lazy<Mutex<Option<Duration>>> = Lazy::new(|| Mutex::new(None));

#[doc(hidden)]
pub static STIME_ACTIVE: Lazy<bool> = Lazy::new(|| std::env::var("STIME").is_ok());

#[doc(hidden)]
#[macro_export]
macro_rules! rtry {
    ($e: expr) => {{
        use $crate::{advanced::*, *};
        if let Err(e) =
            (|| -> ::std::result::Result<(), ::std::boxed::Box<dyn ::std::error::Error>> { $e })()
        {
            panic!("stime failed: {}", e);
        }
    }};
}

/// Start the timer, consecutive calls to [check] will print the elapsed time (and the delta between checks)
///
/// Calling `start` again will restart the timer
///
/// `start` can accept an optional message that implement [std::fmt::Display] to show, if no message is given it will display `file_name:call_line` instead
///
/// @modifier can be used to specify output target, example: start!(@std::io::stdout());
#[macro_export]
macro_rules! start {
    () => {
        start!(concat!(file!(), ":", line!()));
    };
    (@$target: expr) => {
        start!(@target, concat!(file!(), ":", line!()));
    };
    ($msg: expr) => {
        $crate::start!(@::std::io::stderr(), $msg);
    };
    (@$target: expr, $msg: expr) => {
       $crate::rtry!({
            use ::std::io::Write;
            if !&*STIME_ACTIVE {
                return Ok(())
            }
            *CHRONO.lock()? = ::std::time::Instant::now();
            *LAST_DURATION.lock()? = None;
            let mut target = $target;
            writeln!(&mut target, "{} {}", "Starting".red().bold(), $msg.light_blue().italic())?;
            *OUTPUT_TARGET.get() = Box::new(target);
            Ok(())
        });
    };
}

/// Prints the elapsed time since the last call to [start] (and the delta between checks)
///
/// If [start] was not called yet it will print the elapsed time from the program start
///
/// `check` can accept an optional message that implement [std::fmt::Display] to show, if no message is given it will display `file_name:call_line` instead
#[macro_export]
macro_rules! check {
    () => {
        check!(concat!(file!(), ":", line!()));
    };
    ($msg: expr) => {
        $crate::rtry!({
            if !&*STIME_ACTIVE {
                return Ok(());
            }
            let total_time = CHRONO.lock()?.elapsed();
            let delta = if let Some(last_dur) = *LAST_DURATION.lock()? {
                total_time - last_dur
            } else {
                total_time
            };
            *LAST_DURATION.lock()? = Some(total_time);

            writeln!(
                OUTPUT_TARGET.get(),
                //[T  ti  /  D  ti]  msg
                "{}{} {} {} {} {}{} {}",
                "[".light_blue(),
                "TotalTime:".bold(),
                FDur(total_time),
                "/".light_blue(),
                "DeltaTime:".bold(),
                FDur(delta),
                "]".light_blue(),
                $msg.light_blue().italic()
            )
            .map_err(Into::into)
        });
    };
}

/// Convenient utilities for advanced use-cases
pub mod advanced {
    use crate::FDur;
    use once_cell::sync::Lazy;
    use scolor::ColorExt;
    use std::{
        io,
        sync::{Arc, Mutex, MutexGuard},
        time::Instant,
    };

    /// The output target of all logging functions, it defaults to stderr
    pub static OUTPUT_TARGET: Lazy<Target> = Lazy::new(Target::new);

    /// The output target of all logging functions, it defaults to stderr
    pub struct Target {
        inner: Mutex<Box<dyn std::io::Write + Send>>,
    }
    impl Target {
        fn new() -> Self {
            Self {
                inner: Mutex::new(Box::new(std::io::stderr())),
            }
        }
        #[doc(hidden)]
        pub fn get(&self) -> MutexGuard<Box<dyn std::io::Write + Send>> {
            self.inner.lock().unwrap()
        }
        /// Set the output target of logging functions
        pub fn set(&mut self, target: impl std::io::Write + Send + 'static) {
            *self.get() = Box::new(target);
        }
        /// Reset the output target of logging functions to stderr
        pub fn reset(&self) {
            *self.get() = Box::new(std::io::stderr());
        }
    }

    /// Time a block of code
    ///
    /// The timer starts immediately when this function is called
    ///
    /// Its ends when the guard it returns is dropped
    pub fn time_it(msg: &'static str) -> impl Drop {
        struct TimeIt {
            msg: &'static str,
            start: Instant,
        }
        impl Drop for TimeIt {
            fn drop(&mut self) {
                let end = Instant::now();
                let dur = end.duration_since(self.start);
                let _ = writeln!(
                    OUTPUT_TARGET.get(),
                    "{}: {}",
                    self.msg.yellow().italic(),
                    FDur(dur)
                );
            }
        }
        TimeIt {
            start: Instant::now(),
            msg,
        }
    }

    /// Convenient custom log wrapper
    ///
    /// It wraps an Arc so it can be cloned freely
    #[derive(Default)]
    pub struct CustomLog<W> {
        log: Arc<Mutex<W>>,
    }
    impl<W> Clone for CustomLog<W> {
        fn clone(&self) -> Self {
            Self {
                log: self.log.clone(),
            }
        }
    }
    impl<W> CustomLog<W> {
        fn lock(&self) -> MutexGuard<W> {
            self.log.lock().unwrap()
        }
    }
    impl<W: io::Read> CustomLog<W> {
        /// Create a CustomLog from a custom type
        pub fn new(log: W) -> Self {
            Self {
                log: Arc::new(Mutex::new(log)),
            }
        }
        /// Read The log
        pub fn read(&self) -> io::Result<String> {
            let mut s = String::new();
            self.lock().read_to_string(&mut s)?;
            Ok(s)
        }
    }
    impl<W: io::Write> io::Write for CustomLog<W> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.lock().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.lock().flush()
        }
    }
}

#[doc(hidden)]
pub struct FDur(pub std::time::Duration);
impl std::fmt::Display for FDur {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.0.as_secs() != 0 {
            write!(f, "{} {}", self.0.as_secs().red(), "s".red())
        } else if self.0.as_millis() != 0 {
            write!(f, "{} {}", self.0.as_millis().yellow(), "ms".yellow())
        } else if self.0.as_micros() != 0 {
            write!(f, "{} {}", self.0.as_micros().green(), "us".green())
        } else {
            write!(
                f,
                "{} {}",
                self.0.as_nanos().rgb_fg(255, 255, 255),
                "ns".rgb_fg(255, 255, 255)
            )
        }
    }
}
