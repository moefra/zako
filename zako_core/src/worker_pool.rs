use std::thread;

use futures::channel::oneshot;
use zako_cancel::CancelToken;

use crate::worker::WorkerBehavior;

#[derive(Debug)]
pub struct ActorPool<B: WorkerBehavior> {
    sender: flume::Sender<Job<B::Input, B::Output>>,
}

// 内部包装结构：携带了返回用的 oneshot
struct Job<I, O> {
    input: I,
    resp: oneshot::Sender<O>,
    cancel_token: CancelToken,
}

impl<B: WorkerBehavior> Clone for ActorPool<B> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<B: WorkerBehavior> ActorPool<B> {
    /// Create a new pool
    /// concurrency: count of cpu
    pub fn new(machine_concurrency: usize) -> Self {
        let (tx, rx) = flume::bounded(machine_concurrency * 2);

        for _ in 0..machine_concurrency {
            let rx = rx.clone();

            thread::spawn(move || {
                let mut state = B::init();

                while let Ok(job) = rx.recv() {
                    let Job {
                        input,
                        resp,
                        cancel_token,
                    } = job;

                    let output = B::process(&mut state, input, cancel_token);

                    let _ = resp.send(output);
                }

                B::clean(state);
            });
        }

        Self { sender: tx }
    }

    /// Submit a task to the pool and await the result.
    pub async fn submit(
        &self,
        input: B::Input,
        cancel_token: CancelToken,
    ) -> Result<B::Output, String> {
        let (tx, rx) = oneshot::channel();

        let job = Job {
            input,
            resp: tx,
            cancel_token,
        };

        self.sender
            .send_async(job)
            .await
            .map_err(|_| "Worker pool closed".to_string())?;

        rx.await
            .map_err(|_| "Task cancelled or worker panicked".to_string())
    }
}
