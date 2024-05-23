mod round_robin;
mod simple;

pub use round_robin::RRQueue;
pub use simple::SimpleQueue;

use std::pin::Pin;

use futures::Future;

use crate::ContextError;

/// An async task which is executed with a mutable reference to state.
pub trait Task<S: ?Sized, R>: for<'a> FnOnce(&'a mut S) -> TaskFut<'a, R> + Send + 'static {}

impl<S, R, T> Task<S, R> for T where T: for<'a> FnOnce(&'a mut S) -> TaskFut<'a, R> + Send + 'static {}

/// A future that yields the output of a task.
pub type TaskFut<'a, R> = Pin<Box<dyn Future<Output = R> + Send + 'a>>;

/// A queue of tasks which can be executed concurrently with access to shared state.
pub trait Queue<S: ?Sized, R> {
    /// Pushes a task into the queue.
    fn push<F>(&mut self, f: F)
    where
        F: Task<S, R>;

    /// Waits until all the tasks in the queue are completed.
    fn wait(&mut self) -> impl Future<Output = Result<Vec<R>, ContextError>> + Send;
}
