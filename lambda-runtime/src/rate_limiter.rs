//! Thread-safe, process-local rate limiting for infrequent runtime events.

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

/// Allows an operation at most once during each configured interval.
///
/// A `RateLimiter` is intended to be shared by concurrent runtime tasks. When
/// stored in a `static`, it is initialized once per Lambda execution environment
/// and retains its state across warm invocations. A new cold-started environment
/// receives a new limiter.
pub(crate) struct RateLimiter {
    /// Minimum duration between allowed operations.
    interval: Duration,
    /// Timestamp of the most recent allowed operation.
    last_allowed: Mutex<Option<Instant>>,
}

impl RateLimiter {
    /// Creates a rate limiter with the specified minimum interval.
    pub(crate) const fn new(interval: Duration) -> RateLimiter {
        RateLimiter {
            interval,
            last_allowed: Mutex::new(None),
        }
    }

    /// Returns the minimum duration between allowed operations.
    pub(crate) const fn interval(&self) -> Duration {
        self.interval
    }

    ///
    /// The first call is allowed. Subsequent calls are rejected until the
    /// configured interval has elapsed since the previous allowed call.
    /// Concurrent callers are serialized while checking and updating the last
    /// allowed timestamp so only one caller crosses the interval boundary.
    pub(crate) fn allow(&self) -> bool {
        let mut last_allowed = match self.last_allowed.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // The limiter state is disposable, so reset it and recover instead of
                // allowing a poisoned mutex to crash the runtime or suppress future logs.
                let mut guard = poisoned.into_inner();
                *guard = None;
                self.last_allowed.clear_poison();
                guard
            }
        };

        if last_allowed
            .as_ref()
            .is_some_and(|value| value.elapsed() < self.interval)
        {
            return false;
        }

        *last_allowed = Some(Instant::now());

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        panic::{catch_unwind, AssertUnwindSafe},
        thread,
    };

    #[test]
    fn allows_first_call() {
        let limiter = RateLimiter::new(Duration::from_secs(60));

        assert!(limiter.allow());
    }

    #[test]
    fn rejects_calls_inside_interval() {
        let limiter = RateLimiter::new(Duration::from_secs(60));

        assert!(limiter.allow());
        assert!(!limiter.allow());
    }

    #[test]
    fn allows_call_after_interval() {
        let limiter = RateLimiter::new(Duration::from_millis(10));

        assert!(limiter.allow());
        thread::sleep(Duration::from_millis(15));

        assert!(limiter.allow());
    }

    #[test]
    fn recovers_from_poisoned_mutex() {
        let limiter = RateLimiter::new(Duration::from_secs(60));

        let _ = catch_unwind(AssertUnwindSafe(|| {
            let _guard = limiter.last_allowed.lock().unwrap();
            panic!("poison the limiter mutex");
        }));

        assert!(limiter.allow());
        assert!(!limiter.allow());
    }
}
