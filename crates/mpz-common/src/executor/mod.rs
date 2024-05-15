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

    use futures::{Future, FutureExt, TryFutureExt};
    use serio::{
        channel::{duplex, MemoryDuplex},
        codec::Bincode,
    };
    use uid_mux::{
        test_utils::test_yamux_pair_framed,
        yamux::{ConnectionError, YamuxCtrl},
        FramedMux, FramedUidMux,
    };

    use crate::ThreadId;

    use super::*;

    /// Creates a pair of single-threaded executors with memory I/O channels.
    pub fn test_st_executor(
        io_buffer: usize,
    ) -> (STExecutor<MemoryDuplex>, STExecutor<MemoryDuplex>) {
        let (io_0, io_1) = duplex(io_buffer);

        (STExecutor::new(io_0), STExecutor::new(io_1))
    }

    /// Test multi-threaded executor.
    pub type TestMTExecutor = MTExecutor<
        FramedMux<YamuxCtrl, Bincode>,
        <FramedMux<YamuxCtrl, Bincode> as FramedUidMux<ThreadId>>::Framed,
    >;

    /// Creates a pair of multi-threaded executors with yamux I/O channels.
    pub fn test_mt_executor(
        io_buffer: usize,
    ) -> (
        (TestMTExecutor, TestMTExecutor),
        impl Future<Output = Result<(), ConnectionError>>,
    ) {
        let ((mux_0, fut_0), (mux_1, fut_1)) = test_yamux_pair_framed(io_buffer, Bincode);

        let mut fut_io =
            futures::future::try_join(fut_0.into_future(), fut_1.into_future()).map_ok(|_| ());

        let (ctx_0, ctx_1) = futures::executor::block_on(async {
            let fut_exec =
                futures::future::try_join(MTExecutor::new(mux_0), MTExecutor::new(mux_1));
            futures::select! {
                ctx = fut_exec.fuse() => ctx.unwrap(),
                _ = (&mut fut_io).fuse() => panic!(),
            }
        });

        ((ctx_0, ctx_1), fut_io)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_create_mt_executor() {
            _ = test_mt_executor(1024);
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub use test_utils::*;
