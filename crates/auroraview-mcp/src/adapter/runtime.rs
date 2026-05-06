//! Runtime bridge for synchronous adapter traits over async CDP calls.

use tokio::runtime::{Handle, Runtime};

#[derive(Debug)]
pub(super) enum AdapterRuntime {
    /// We are inside a running tokio runtime — reuse it.
    Borrowed(Handle),
    /// No ambient runtime; we own a multi-thread runtime.
    Owned(Runtime),
}

impl AdapterRuntime {
    pub(super) fn current_or_owned() -> std::io::Result<Self> {
        if let Ok(handle) = Handle::try_current() {
            Ok(Self::Borrowed(handle))
        } else {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .worker_threads(1)
                .thread_name("auroraview-mcp")
                .build()?;
            Ok(Self::Owned(rt))
        }
    }

    pub(super) fn block_on<F: std::future::Future>(&self, fut: F) -> F::Output {
        match self {
            Self::Borrowed(h) => {
                // `Handle::block_on` panics if we're currently on a runtime
                // thread, so we offload to a temporary blocking task.
                tokio::task::block_in_place(|| h.block_on(fut))
            }
            Self::Owned(rt) => rt.block_on(fut),
        }
    }
}
