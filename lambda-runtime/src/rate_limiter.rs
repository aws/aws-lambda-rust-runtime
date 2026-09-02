//! Internal call-site rate limiting for infrequent runtime events.

use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

/// Returns monotonic process-relative time for the rate limiter.
pub(crate) fn time_since_epoch() -> Duration {
    static EPOCH: OnceLock<Instant> = OnceLock::new();

    Instant::now().duration_since(*EPOCH.get_or_init(Instant::now))
}

/// Evaluates a call at most once per interval at each macro call site.
///
/// The limiter state is local to the call site and persists for the lifetime of
/// the process. It is therefore shared across warm invocations in one Lambda
/// execution environment and reset by a cold start.
macro_rules! rate_limited {
    ($interval:expr, $call:expr) => {{
        use std::sync::atomic::{AtomicU64, Ordering};

        static NEXT_CALL: AtomicU64 = AtomicU64::new(u64::MIN);
        let interval: std::time::Duration = $interval;
        let time = $crate::rate_limiter::time_since_epoch();
        let next = NEXT_CALL.load(Ordering::Relaxed);

        if next <= time.as_secs() {
            let new_next = time.checked_add(interval).unwrap_or(std::time::Duration::MAX).as_secs();

            if NEXT_CALL
                .compare_exchange(next, new_next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                $call;
            }
        }
    }};
}

pub(crate) use rate_limited;

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        thread,
    };

    #[test]
    fn allows_first_call_and_rejects_calls_inside_interval() {
        let mut calls = 0;

        for _ in 0..2 {
            rate_limited!(Duration::from_secs(60), {
                calls += 1;
            });
        }

        assert_eq!(calls, 1);
    }

    #[test]
    fn allows_only_one_concurrent_call() {
        let calls = Arc::new(AtomicUsize::new(0));
        let handles = (0..8)
            .map(|_| {
                let calls = Arc::clone(&calls);
                thread::spawn(move || {
                    rate_limited!(Duration::from_secs(60), {
                        calls.fetch_add(1, Ordering::Relaxed);
                    });
                })
            })
            .collect::<Vec<_>>();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
}
