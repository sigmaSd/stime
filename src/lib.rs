#![warn(missing_docs)]
//!Chrono
//!
//!Easy API to time code.
//!
//!It only exposes 2 macros:
//!- [start] => start the timer
//!- [check] => print the elapsed duration since the last start (and the delta between checks)
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

/// Start the timer, consecutive calls to [check] will print the elapsed time (and the delta between checks)
///
/// Calling `start` again will restart the timer
///
/// `start` can accept an optional message that implement [std::fmt::Display] to show, if no message is given it will display `file_name:call_line` instead
#[macro_export]
macro_rules! start {
    () => {
        start!(concat!(file!(), ":", line!()));
    };
    ($msg: expr) => {{
        use stime::*;
        let now = std::time::Instant::now();
        *CHRONO.lock().unwrap() = now;
        *LAST_DURATION.lock().unwrap() = None;
        let msg = $msg;
        eprintln!("{} {}", "Starting".red().bold(), msg.light_blue().italic());
    }};
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
    ($msg: expr) => {{
        use stime::*;
        let total_time = CHRONO.lock().unwrap().elapsed();
        let delta = if let Some(last_dur) = *LAST_DURATION.lock().unwrap() {
            total_time - last_dur
        } else {
            total_time
        };
        *LAST_DURATION.lock().unwrap() = Some(total_time);

        struct FDur(std::time::Duration);
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

        let msg = $msg;
        eprintln!(
            //[T  ti  /  D  ti]  msg
            "{}{} {} {} {} {}{} {}",
            "[".light_blue(),
            "TotalTime:".bold(),
            FDur(total_time),
            "/".light_blue(),
            "DeltaTime:".bold(),
            FDur(delta),
            "]".light_blue(),
            msg.light_blue().italic()
        );
    }};
}
