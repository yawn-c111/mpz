use std::collections::VecDeque;

use futures::Future;

use crate::{
    queue::{Queue, Task},
    ContextError,
};

/// A queue that executes tasks one at a time in FIFO order.
pub struct SimpleQueue<'a, S: ?Sized, R> {
    state: &'a mut S,
    queue: VecDeque<Box<dyn Task<S, R>>>,
}

impl<'a, S: ?Sized, R> SimpleQueue<'a, S, R> {
    /// Creates a new simple queue.
    pub fn new(state: &'a mut S) -> Self {
        Self {
            state,
            queue: VecDeque::new(),
        }
    }
}

impl<'a, S: ?Sized, R> Queue<S, R> for SimpleQueue<'a, S, R>
where
    S: Send,
    R: Send + 'static,
{
    fn push<F>(&mut self, f: F)
    where
        F: Task<S, R>,
    {
        self.queue.push_back(Box::new(f));
    }

    fn wait(&mut self) -> impl Future<Output = Result<Vec<R>, ContextError>> + Send {
        async {
            let mut results = Vec::with_capacity(self.queue.len());
            while let Some(task) = self.queue.pop_front() {
                results.push(task(self.state).await);
            }

            Ok(results)
        }
    }
}
