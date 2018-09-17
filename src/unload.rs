use super::errors;
use super::nr;
use errno;
#[cfg(feature = "async")]
use futures::prelude::*;
use libc;

/// Module unloader.
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

    /// Unload module by name, optionally blocking until completion.
    ///
    /// This is a synchronous method that can optionally block (putting
    /// the process in D state) while waiting for module reference count
    /// to be 0 for clean unloading (unless forced, or when `blocking`
    /// parameter is `false`).
    /// It returns once unload is complete or an error happened.
    pub fn blocking_unload<S: AsRef<str>>(&self, modname: S, blocking: bool) -> errors::Result<()> {
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

    #[cfg(feature = "async")]
    /// Unload module by name, asynchronously.
    pub fn async_unload<S: AsRef<str>>(
        &self,
        modname: S,
    ) -> Box<Future<Item = (), Error = errors::Error>> {
        let flags = {
            let ff = if self.force { libc::O_TRUNC } else { 0 };
            ff | libc::O_NONBLOCK
        };
        let unloader = UnloadTask {
            flags,
            modname: modname.as_ref().to_string(),
        };
        Box::new(unloader)
    }
}

#[cfg(feature = "async")]
pub(crate) struct UnloadTask {
    flags: libc::c_int,
    modname: String,
}

#[cfg(feature = "async")]
impl Future for UnloadTask {
    type Item = ();
    type Error = errors::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use futures;
        // UNSAFE(lucab): required syscall, all parameters are immutable.
        let r = unsafe { libc::syscall(nr::DELETE_MODULE, self.modname.as_ptr(), self.flags) };

        // Successfully unloaded.
        if r == 0 {
            return Ok(Async::Ready(()));
        }

        // Module is busy, keep polling later.
        let num = errno::errno();
        if num.0 == libc::EWOULDBLOCK {
            // TODO(lucab): rate-limit this.
            futures::task::current().notify();
            return Ok(Async::NotReady);
        }

        // Any other generic error, bubble this up.
        Err(errors::Error::from_kind(errors::ErrorKind::Sys(num))
            .chain_err(|| "async delete_module error"))
    }
}
