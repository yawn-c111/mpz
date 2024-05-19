//! Executors.

mod dummy;
mod mt;
mod st;

pub use dummy::{DummyExecutor, DummyIo};
pub use mt::MTExecutor;
pub use st::STExecutor;

#[cfg(any(test, feature = "test-utils"))]
mod test_utils {
    use std::future::IntoFuture;

    use futures::{Future, TryFutureExt};
    use serio::{
        channel::{duplex, MemoryDuplex},
        codec::Bincode,
    };
    use uid_mux::{
        test_utils::test_yamux_pair_framed,
        yamux::{ConnectionError, YamuxCtrl},
        FramedMux,
    };

    use super::*;

    /// Creates a pair of single-threaded executors with memory I/O channels.
    pub fn test_st_executor(
        io_buffer: usize,
    ) -> (STExecutor<MemoryDuplex>, STExecutor<MemoryDuplex>) {
        let (io_0, io_1) = duplex(io_buffer);

        (STExecutor::new(io_0), STExecutor::new(io_1))
    }

    /// Test multi-threaded executor.
    pub type TestMTExecutor = MTExecutor<FramedMux<YamuxCtrl, Bincode>>;

    /// Creates a pair of multi-threaded executors with yamux I/O channels.
    pub fn test_mt_executor(
        io_buffer: usize,
    ) -> (
        (TestMTExecutor, TestMTExecutor),
        impl Future<Output = Result<(), ConnectionError>>,
    ) {
        let ((mux_0, fut_0), (mux_1, fut_1)) = test_yamux_pair_framed(io_buffer, Bincode);

        let fut_io =
            futures::future::try_join(fut_0.into_future(), fut_1.into_future()).map_ok(|_| ());

        let exec_0 = MTExecutor::new(mux_0);
        let exec_1 = MTExecutor::new(mux_1);

        ((exec_0, exec_1), fut_io)
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub use test_utils::*;
