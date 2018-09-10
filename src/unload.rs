use super::errors;
use super::nr;
use errno;
use libc;

/// Module unloader.
#[derive(Debug)]
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
                    .chain_err(|| "delete_module error"),
            ),
        }
    }

    // TODO(lucab): implement async_unload with futures-0.3
}
