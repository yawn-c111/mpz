use std::{collections::VecDeque, mem};

use futures::Future;

use crate::{
    queue::{Queue, Task},
    ContextError,
};

/// A queue that load balances tasks in a round-robin fashion.
pub struct RRQueue<'a, L, R> {
    lanes: &'a mut [L],
    queue: VecDeque<Box<dyn Task<L, R>>>,
}

impl<'a, L, R> RRQueue<'a, L, R> {
    /// Creates a new round-robin queue.
    pub fn new(lanes: &'a mut [L]) -> Self {
        Self {
            lanes,
            queue: VecDeque::new(),
        }
    }
}

impl<'a, L, R> Queue<L, R> for RRQueue<'a, L, R>
where
    L: Send,
    R: Send,
{
    fn push<F>(&mut self, f: F)
    where
        F: Task<L, R>,
    {
        self.queue.push_back(Box::new(f));
    }

    fn wait(&mut self) -> impl Future<Output = Result<Vec<R>, ContextError>> + Send {
        let lane_count = self.lanes.len();
        let task_count = self.queue.len();

        let mut lanes: Vec<_> = (0..lane_count)
            .map(|_| Vec::with_capacity((task_count / lane_count) + (task_count % lane_count)))
            .collect();

        // Distribute tasks evenly across lanes.
        for (i, task) in mem::take(&mut self.queue).into_iter().enumerate() {
            lanes[i % lane_count].push(task);
        }

        // Create futures which process each lane with the corresponding state.
        let lane_futs: Vec<_> = lanes
            .into_iter()
            .zip(self.lanes.iter_mut())
            .map(|(lane, ctx)| async move {
                let mut results = Vec::with_capacity(lane.len());
                for task in lane {
                    results.push(task(ctx).await);
                }
                results.reverse();
                results
            })
            .collect();

        async move {
            let mut lane_results = futures::future::join_all(lane_futs).await;
            let mut results = Vec::with_capacity(task_count);

            // Interleave the results from each lane.
            for lane_idx in (0..lane_count).cycle() {
                if let Some(result) = lane_results[lane_idx].pop() {
                    results.push(result);
                } else {
                    break;
                }
            }

            Ok(results)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{executor::test_mt_executor, Context};

    use super::*;

    #[tokio::test]
    async fn test_rr_queue() {
        let ((mut exec_a, _), io_fut) = test_mt_executor(1024);

        tokio::spawn(io_fut);

        let mut ctx = vec![
            exec_a.new_thread().await.unwrap(),
            exec_a.new_thread().await.unwrap(),
            exec_a.new_thread().await.unwrap(),
        ];

        let mut queue = RRQueue::new(&mut ctx);

        queue.push(|ctx| {
            Box::pin(async {
                _ = ctx.id();
                0
            })
        });
        queue.push(|ctx| {
            Box::pin(async {
                _ = ctx.id();
                1
            })
        });
        queue.push(|ctx| {
            Box::pin(async {
                _ = ctx.id();
                2
            })
        });
        queue.push(|ctx| {
            Box::pin(async {
                _ = ctx.id();
                3
            })
        });

        let results = queue.wait().await.unwrap();

        assert_eq!(results.len(), 4);
        // Results are in the order they were pushed.
        assert_eq!(results, vec![0, 1, 2, 3]);
    }
}
