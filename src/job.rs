//! A trait representing a job.

use std::str::FromStr;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;

use error::{Error, ErrorKind, Result};

/// A job and its related metadata (name, queue, timeout, etc.)
///
/// In most cases, you should be deriving this trait instead of implementing it manually yourself.
///
/// # Examples
///
/// Using the provided defaults:
///
/// ```rust
/// #[macro_use]
/// extern crate batch;
/// #[macro_use]
/// extern crate lazy_static;
/// #[macro_use]
/// extern crate serde;
///
/// #[derive(Deserialize, Serialize, Job)]
/// #[job_routing_key = "emails"]
/// struct SendConfirmationEmail;
///
/// #
/// # fn main() {}
/// ```
///
/// Overriding the provided defaults:
///
/// ```rust
/// #[macro_use]
/// extern crate batch;
/// #[macro_use]
/// extern crate lazy_static;
/// #[macro_use]
/// extern crate serde;
///
/// struct App;
///
/// #[derive(Deserialize, Serialize, Job)]
/// #[job_name = "batch-rs:send-password-reset-email"]
/// #[job_routing_key = "emails"]
/// #[job_timeout = "120"]
/// #[job_retries = "0"]
/// struct SendPasswordResetEmail;
///
/// #
/// # fn main() {}
/// ```
pub trait Job: DeserializeOwned + Serialize {
    /// A should-be-unique human-readable ID for this job.
    fn name() -> &'static str;

    /// The exchange the job will be published to.
    fn exchange() -> &'static str;

    /// The routing key associated to this job.
    fn routing_key() -> &'static str;

    /// The number of times this job must be retried in case of error.
    fn retries() -> u32;

    /// An optional duration representing the time allowed for this job's handler to complete.
    fn timeout() -> Option<Duration>;

    /// The priority associated to this job.
    fn priority() -> Priority;
}

/// The different priorities that can be assigned to a `Job`.
///
/// The default value is `Priority::Normal`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// The lowest available priority for a job.
    Trivial,
    /// A lower priority than `Priority::Normal` but higher than `Priority::Trivial`.
    Low,
    /// The default priority for a job.
    Normal,
    /// A higher priority than `Priority::Normal` but higher than `Priority::Critical`.
    High,
    /// The highest available priority for a job.
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl FromStr for Priority {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "trivial" => Ok(Priority::Trivial),
            "low" => Ok(Priority::Low),
            "normal" => Ok(Priority::Normal),
            "high" => Ok(Priority::High),
            "critical" => Ok(Priority::Critical),
            _ => Err(ErrorKind::InvalidPriority)?,
        }
    }
}

impl Priority {
    /// Return the priority as a `u8` ranging from 0 to 4.
    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            Priority::Trivial => 0,
            Priority::Low => 1,
            Priority::Normal => 2,
            Priority::High => 3,
            Priority::Critical => 4,
        }
    }
}

/// The different states a `Job` can be in.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Status {
    /// The job was created but it wasn't sent/received yet.
    Pending,
    /// The job was received by a worker that started executing it.
    Started,
    /// The job completed successfully.
    Success,
    /// The job didn't complete successfully, see attached `Failure` cause.
    Failed(Failure),
}

/// Stores the reason for a job failure.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Failure {
    /// The job handler returned an error.
    Error,
    /// The job didn't complete in time.
    Timeout,
    /// The job crashed (panic, segfault, etc.) while executing.
    Crash,
}

/// The `Perform` trait allow marking a `Job` as executable.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate batch;
/// #[macro_use]
/// extern crate lazy_static;
/// #[macro_use]
/// extern crate serde;
///
/// use batch::Perform;
///
/// #[derive(Serialize, Deserialize, Job)]
/// #[job_routing_key = "emails"]
/// struct SendPasswordResetEmail;
///
/// impl Perform for SendPasswordResetEmail {
///     type Context = ();
///
///     fn perform(&self, _ctx: Self::Context) {
///         println!("Sending password reset email...");
///     }
/// }
///
/// # fn main() {}
/// ```
pub trait Perform {
    /// The type of the context value that will be given to this job's handler.
    type Context;

    /// Perform the job's duty.
    fn perform(&self, Self::Context);
}
