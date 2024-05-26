//! Executors.

mod dummy;
mod mt;
mod st;

pub use dummy::{DummyExecutor, DummyIo};
pub use mt::{MTContext, MTExecutor};
pub use st::STExecutor;

#[cfg(any(test, feature = "test-utils"))]
mod test_utils {
    use serio::{
        channel::{duplex, MemoryDuplex},
        codec::Bincode,
    };
    use uid_mux::{
        test_utils::{test_framed_mux, TestFramedMux},
        yamux::YamuxCtrl,
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

    /// Creates a pair of multi-threaded executors with memory I/O channels.
    pub fn test_mt_executor(
        io_buffer: usize,
    ) -> (MTExecutor<TestFramedMux>, MTExecutor<TestFramedMux>) {
        let (mux_0, mux_1) = test_framed_mux(io_buffer);

        (MTExecutor::new(mux_0, 8), MTExecutor::new(mux_1, 8))
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub use test_utils::*;
