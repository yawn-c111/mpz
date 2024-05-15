use async_trait::async_trait;
use scoped_futures::ScopedBoxFuture;
use serio::IoDuplex;
use uid_mux::FramedUidMux;

use crate::{
    context::{ContextError, ErrorKind},
    Context, ThreadId,
};

/// A multi-threaded executor.
#[derive(Debug)]
pub struct MTExecutor<M, Io> {
    id: ThreadId,
    mux: M,
    io: Io,
    // TODO: Support multiple children. Right now this is simpler to implement,
    // and our `Context` trait only exposes joining two futures. Eventually we will
    // support an API similar to `FuturesOrdered`.
    child: Option<Box<Self>>,
}

impl<M, Io> MTExecutor<M, Io> {
    fn set_child(&mut self, child: Box<Self>) {
        self.child = Some(child);
    }
}

impl<M, Io> MTExecutor<M, Io>
where
    M: FramedUidMux<ThreadId, Framed = Io> + Clone,
    M::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
{
    /// Creates a new multi-threaded executor.
    pub async fn new(mux: M) -> Result<Self, ContextError> {
        let id = ThreadId::default();
        let io = mux
            .open_framed(&id)
            .await
            .map_err(|e| ContextError::new_with_source(ErrorKind::Mux, e))?;

        Ok(Self {
            id,
            mux,
            io,
            child: None,
        })
    }

    async fn fork(&mut self) -> Result<Box<Self>, ContextError> {
        // Forking a thread context is only performed once, afterwhich the child ctx
        // is stored for later use.

        if let Some(child) = self.child.take() {
            return Ok(child);
        }

        let child_id = self.id.fork();
        let io = self
            .mux
            .open_framed(&child_id)
            .await
            .map_err(|e| ContextError::new_with_source(ErrorKind::Mux, e))?;

        let child = Self {
            id: child_id,
            mux: self.mux.clone(),
            io,
            child: None,
        };

        Ok(Box::new(child))
    }
}

#[async_trait]
impl<M, Io> Context for MTExecutor<M, Io>
where
    M: FramedUidMux<ThreadId, Framed = Io> + Clone + Send + Sync,
    M::Error: Into<Box<dyn std::error::Error + Send + Sync + 'static>>,
    Io: IoDuplex + Send + Sync + Unpin + 'static,
{
    type Io = Io;

    fn id(&self) -> &ThreadId {
        &self.id
    }

    fn io_mut(&mut self) -> &mut Self::Io {
        &mut self.io
    }

    async fn join<'a, A, B, RA, RB>(&'a mut self, a: A, b: B) -> Result<(RA, RB), ContextError>
    where
        A: for<'b> FnOnce(&'b mut Self) -> ScopedBoxFuture<'a, 'b, RA> + Send + 'a,
        B: for<'b> FnOnce(&'b mut Self) -> ScopedBoxFuture<'a, 'b, RB> + Send + 'a,
        RA: Send + 'a,
        RB: Send + 'a,
    {
        let mut child = self.fork().await?;
        let output = futures::join!(a(self), b(&mut child));
        self.set_child(child);
        Ok(output)
    }

    async fn try_join<'a, A, B, RA, RB, E>(
        &'a mut self,
        a: A,
        b: B,
    ) -> Result<Result<(RA, RB), E>, ContextError>
    where
        A: for<'b> FnOnce(&'b mut Self) -> ScopedBoxFuture<'a, 'b, Result<RA, E>> + Send + 'a,
        B: for<'b> FnOnce(&'b mut Self) -> ScopedBoxFuture<'a, 'b, Result<RB, E>> + Send + 'a,
        RA: Send + 'a,
        RB: Send + 'a,
        E: Send + 'a,
    {
        let mut child = self.fork().await?;
        let output = futures::try_join!(a(self), b(&mut child));
        self.set_child(child);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::future::IntoFuture;

    use crate::join;
    use serio::codec::Bincode;
    use uid_mux::test_utils::test_yamux_pair_framed;

    use super::*;

    #[derive(Debug, Default)]
    struct LifetimeTest {
        a: ThreadId,
        b: ThreadId,
    }

    impl LifetimeTest {
        // This test is to ensure that the compiler is satisfied with the lifetimes
        // of the async closures passed to `join`.
        async fn foo<Ctx: Context>(&mut self, ctx: &mut Ctx) {
            let a = &mut self.a;
            let b = &mut self.b;

            join! {
                ctx,
                async {
                    *a = ctx.id().clone();
                },
                async {
                    *b = ctx.id().clone();
                }
            }
            .unwrap();

            // Make sure we can mutate the fields after borrowing them in the async closures.
            self.a = ThreadId::default();
            self.b = ThreadId::default();
        }
    }

    #[tokio::test]
    async fn test_mt_executor_join() {
        tracing_subscriber::fmt::init();
        let ((mux_a, fut_a), (mux_b, fut_b)) = test_yamux_pair_framed(1024, Bincode);

        tokio::spawn(async move {
            futures::try_join!(fut_a.into_future(), fut_b.into_future()).unwrap();
        });

        let (mut ctx_a, mut ctx_b) =
            futures::try_join!(MTExecutor::new(mux_a), MTExecutor::new(mux_b)).unwrap();

        let mut test_a = LifetimeTest::default();
        let mut test_b = LifetimeTest::default();

        futures::join!(test_a.foo(&mut ctx_a), test_b.foo(&mut ctx_b));
    }
}
