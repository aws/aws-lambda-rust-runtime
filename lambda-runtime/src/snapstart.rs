//! SnapStart lifecycle support.
//!
//! AWS Lambda SnapStart reduces cold start latency by taking a Firecracker
//! microVM snapshot of the initialized execution environment and restoring from
//! it on subsequent invocations. Resources initialized before the snapshot may
//! become invalid after restore: TCP connections are dropped, credentials may
//! expire, and unique values generated during init would be shared across every
//! restored instance.
//!
//! Implement [`SnapStartResource`] on types that hold such resources and
//! register them with
//! [`Runtime::register_snapstart_resource`](crate::Runtime::register_snapstart_resource).
//! The runtime invokes the registered hooks around the snapshot/restore boundary
//! when SnapStart is enabled (`AWS_LAMBDA_INITIALIZATION_TYPE == "snap-start"`)
//! and does nothing otherwise.
//!
//! # Ordering
//!
//! Resources are orchestrated with stack (LIFO/FIFO) ordering, mirroring how
//! Go's `defer`, middleware unwinding, and Rust destructors work:
//!
//! - [`before_snapshot`](SnapStartResource::before_snapshot) runs in **reverse**
//!   registration order (LIFO) — dependents tear down before their foundations.
//! - [`after_restore`](SnapStartResource::after_restore) runs in **registration**
//!   order (FIFO) — foundations rebuild before the dependents that need them.
//!
//! The rule for users is one line: **register foundations first** (e.g.
//! credentials before the pool that uses them, the pool before the cache that
//! uses it). Teardown and rebuild then both happen in the correct relative order
//! automatically:
//!
//! ```text
//! register:        credentials → pool → cache
//! before_snapshot: cache → pool → credentials   (reverse)
//! after_restore:   credentials → pool → cache    (forward)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use lambda_runtime::{BoxFuture, Error, SnapStartResource};
//!
//! struct DbPool {
//!     // e.g. a connection pool handle
//! }
//!
//! impl SnapStartResource for DbPool {
//!     fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
//!         Box::pin(async move {
//!             // release connections before the snapshot
//!             Ok(())
//!         })
//!     }
//!
//!     fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
//!         Box::pin(async move {
//!             // re-establish connections after restore
//!             Ok(())
//!         })
//!     }
//! }
//! ```

use crate::{BoxFuture, Error};

/// A resource that needs custom logic around the SnapStart snapshot/restore
/// boundary.
///
/// Implement this trait on types holding snapshot-sensitive state (connection
/// pools, credentials, cached DNS, etc.) and register them with
/// [`Runtime::register_snapstart_resource`](crate::Runtime::register_snapstart_resource).
///
/// Both methods default to a no-op, so implementors only override the hook they
/// need. See the [module docs](crate::snapstart) for ordering semantics.
///
/// Each method returns a [`BoxFuture`] (rather than being an `async fn`) so the
/// trait stays object-safe and can be stored as `dyn SnapStartResource` without
/// pulling in the `async-trait` crate. Implementations wrap their body in
/// `Box::pin(async move { .. })`.
pub trait SnapStartResource: Send + Sync {
    /// Called before the VM snapshot is taken.
    ///
    /// Use this to release resources that will not survive the snapshot, such as
    /// open connections or buffered data.
    ///
    /// Runs in reverse registration order (LIFO) across all registered
    /// resources.
    fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async { Ok(()) })
    }

    /// Called after the VM is restored from a snapshot.
    ///
    /// Use this to re-establish resources released in
    /// [`before_snapshot`](Self::before_snapshot), refresh credentials, or
    /// regenerate values that must be unique per execution environment.
    ///
    /// Runs in registration order (FIFO) across all registered resources. If
    /// this returns an error, the runtime reports it to Lambda via
    /// `/restore/error` and the runtime exits.
    fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async { Ok(()) })
    }
}
