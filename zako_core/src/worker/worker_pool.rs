use std::{
    sync::{
        Arc, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use flume::RecvTimeoutError;
use futures::channel::oneshot;
use tokio::sync::broadcast::{self, error::SendError};
use zako_cancel::CancelToken;

use crate::worker::WorkerBehavior;

#[derive(Debug, thiserror::Error)]
pub enum WorkerPoolError {
    #[error("Worker pool started multiple times")]
    StartMultipleTimes,
    #[error("Worker pool not started")]
    NotStarted,
    #[error("Task cancelled or worker panicked")]
    Canceled,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_workers: usize,
    pub min_workers: usize,
    pub idle_timeout: Duration,
}

pub struct WorkerPool<B: WorkerBehavior> {
    sender: flume::Sender<Command<B>>,
    receiver: flume::Receiver<Command<B>>,
    gc_sender: broadcast::Sender<()>,
    context: OnceLock<Arc<B::Context>>,
    config: PoolConfig,
    active_count: Arc<AtomicUsize>,
}

impl<B: WorkerBehavior> std::fmt::Debug for WorkerPool<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerPool")
            .field("config", &self.config)
            .field("active_count", &self.active_count)
            .finish()
    }
}

// 内部包装结构：携带了返回用的 oneshot
struct Job<I, O> {
    input: I,
    resp: oneshot::Sender<O>,
    cancel_token: CancelToken,
}

enum Command<B: WorkerBehavior> {
    _Gc,
    Process(Job<B::Input, B::Output>),
}

impl<B: WorkerBehavior> WorkerPool<B> {
    pub fn start(&self, ctx: Arc<B::Context>) -> Result<(), WorkerPoolError> {
        self.context
            .set(ctx)
            .map_err(|_| WorkerPoolError::StartMultipleTimes)?;
        Ok(())
    }

    fn spawn_thread(&self) {
        let rx = self.receiver.clone();
        let ctx = self.context.get().unwrap().clone();
        let mut gc_rx = self.gc_sender.subscribe();
        let alive_time = self.config.idle_timeout;
        let active_count = self.active_count.clone();

        thread::spawn(move || {
            let mut state = B::init(&ctx);

            // refresh this when process a job
            // but do not refresh when gc
            let mut last_active_time = Instant::now();

            loop {
                if gc_rx.try_recv().is_ok() {
                    B::gc(&mut state);
                }

                let timeout = rx.recv_timeout(Duration::from_secs_f64(0.5));

                if Instant::now().duration_since(last_active_time) > alive_time {
                    break;
                }

                match timeout {
                    Ok(command) => match command {
                        Command::_Gc => B::gc(&mut state),
                        Command::Process(job) => {
                            last_active_time = Instant::now(); // reset the last active time

                            let Job {
                                input,
                                resp,
                                cancel_token,
                            } = job;

                            let output = B::process(&mut state, input, cancel_token);

                            let _ = resp.send(output);
                        }
                    },
                    Err(e) => {
                        if e == RecvTimeoutError::Timeout {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            }

            let _ = active_count.fetch_sub(1, Ordering::Relaxed);
        });
    }

    fn maybe_spawn_worker(&self) {
        let current = self.active_count.load(Ordering::Relaxed);

        // 扩容条件：
        // 1. 当前线程数没达到上限
        // 2. 队列里有积压 (len > 0) 或者 当前没有线程 (current == 0)
        //    (注意：async_channel 的 len() 是近似值，但足够了)
        if current < self.config.max_workers && (current == 0 || !self.sender.is_empty()) {
            // 用乐观锁增加计数，防止多个超过线程上限
            if self
                .active_count
                .compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                self.spawn_thread();
            }
        }
    }

    /// Create a new pool
    /// concurrency: count of cpu
    pub fn new(config: PoolConfig) -> Self {
        let (tx, rx) = flume::bounded(config.max_workers * 2);
        let (gc_tx, _) = broadcast::channel(1);

        Self {
            sender: tx,
            receiver: rx,
            gc_sender: gc_tx,
            active_count: Arc::new(AtomicUsize::new(0)),
            config,
            context: OnceLock::new(),
        }
    }

    /// Submit a task to the pool and await the result.
    pub async fn submit(
        &self,
        input: B::Input,
        cancel_token: CancelToken,
    ) -> Result<B::Output, WorkerPoolError> {
        let (tx, rx) = oneshot::channel();

        let job = Job {
            input,
            resp: tx,
            cancel_token,
        };

        self.sender
            .send(Command::Process(job))
            .map_err(|_| WorkerPoolError::NotStarted)?;

        self.maybe_spawn_worker();

        rx.await.map_err(|_| WorkerPoolError::Canceled)
    }

    pub async fn gc(&self) -> Result<(), SendError<()>> {
        self.gc_sender.send(())?;
        Ok(())
    }
}
