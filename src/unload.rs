use super::errors;
use super::nr;
use errno;
#[cfg(feature = "async")]
use futures::prelude::*;
use libc;
#[cfg(feature = "async")]
use tokio::timer;

/// Module unloader.
///
/// Asynchronous methods can be enabled via the optional `async` feature.
#[derive(Clone, Debug)]
pub struct ModUnloader {
    force: bool,
}

impl Default for ModUnloader {
    fn default() -> Self {
        Self { force: false }
    }
}

impl ModUnloader {
    /// Create a new default `ModLoader`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether a forced unload should be performed.
    ///
    /// A force unload will taint the kernel and can leave the
    /// host in an unstable state, or cause data loss.
    pub fn forced(mut self, force_unload: bool) -> Self {
        self.force = force_unload;
        self
    }

    /// Unload module by name, synchronously.
    ///
    /// If `blocking` is enabled, this can block at syscall level (putting
    /// the process in D state) while waiting for module reference count
    /// to be 0 for clean unloading (unless forced).
    ///
    /// It is usually recommended not to set `blocking`, as the process
    /// cannot be killed while blocked on syscall. Consider using
    /// `unload_async` instead.
    pub fn unload_sync<S: AsRef<str>>(&self, modname: S, blocking: bool) -> errors::Result<()> {
        let flags = match (self.force, blocking) {
            (false, false) => 0,
            (true, false) => libc::O_TRUNC,
            (true, true) => libc::O_TRUNC | libc::O_NONBLOCK,
            (false, true) => libc::O_NONBLOCK,
        };
        // UNSAFE(lucab): required syscall, all parameters are immutable.
        let r = unsafe { libc::syscall(nr::DELETE_MODULE, modname.as_ref().as_ptr(), flags) };
        match r {
            0 => Ok(()),
            _ => Err(
                errors::Error::from_kind(errors::ErrorKind::Sys(errno::errno()))
                    .chain_err(|| "blocking delete_module error"),
            ),
        }
    }

    /// Unload module by name, asynchronously.
    ///
    /// If the module is currently in use, this will continuously retry
    /// unloading at fixed intervals after pausing for the specified
    /// amount of milliseconds.
    /// This requires enabling the `async` optional feature.
    #[cfg(feature = "async")]
    pub fn unload_async<S: AsRef<str>>(
        &self,
        modname: S,
        pause_millis: ::std::num::NonZeroU64,
    ) -> Box<Future<Item = (), Error = errors::Error>> {
        let flags = {
            let ff = if self.force { libc::O_TRUNC } else { 0 };
            ff | libc::O_NONBLOCK
        };
        let pause = ::std::time::Duration::from_millis(pause_millis.get());
        let unloader = UnloadTask {
            flags,
            interval: timer::Interval::new_interval(pause),
            modname: modname.as_ref().to_string(),
        };
        Box::new(unloader)
    }
}

#[cfg(feature = "async")]
pub(crate) struct UnloadTask {
    flags: libc::c_int,
    interval: timer::Interval,
    modname: String,
}

#[cfg(feature = "async")]
impl Future for UnloadTask {
    type Item = ();
    type Error = errors::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use futures::task;

        // Rate-limit to a sane interval, as `delete_module(2)`
        // has no feedback mechanism.
        match self.interval.poll() {
            Ok(Async::Ready(_)) => {}
            Ok(Async::NotReady) => {
                task::current().notify();
                return Ok(Async::NotReady);
            }
            Err(e) => bail!("failed rate-limiting interval: {}", e),
        };

        // UNSAFE(lucab): required syscall, all parameters are immutable.
        let r = unsafe { libc::syscall(nr::DELETE_MODULE, self.modname.as_ptr(), self.flags) };

        // Successfully unloaded.
        if r == 0 {
            return Ok(Async::Ready(()));
        }

        // Module is busy, keep polling later.
        let num = errno::errno();
        if num.0 == libc::EWOULDBLOCK {
            task::current().notify();
            return Ok(Async::NotReady);
        }

        // Any other generic error, bubble this up.
        Err(errors::Error::from_kind(errors::ErrorKind::Sys(num))
            .chain_err(|| "async delete_module error"))
    }
}
