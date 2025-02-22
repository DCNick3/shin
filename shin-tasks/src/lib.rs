use std::{fmt, fmt::Debug, io, num::NonZeroUsize};

use futures_lite::FutureExt as _;
use tracing::debug;
use wasm_thread as thread;

pub struct AsyncTask<T> {
    task: async_task::Task<T>,
}

impl<T> AsyncTask<T> {
    pub fn detach(self) {
        self.task.detach()
    }

    pub fn poll_naive(&mut self) -> Option<T> {
        // this is slightly inefficient as it will end up registering the waker, even though we don't need it
        // however, we would need to write our own task primitive if we want different behavior
        match self
            .task
            .poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
        {
            std::task::Poll::Ready(result) => Some(result),
            std::task::Poll::Pending => None,
        }
    }
}

impl<T> Future for AsyncTask<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.get_mut().task.poll(cx)
    }
}

impl<T> Debug for AsyncTask<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.task.fmt(f)
    }
}

pub struct ComputeTask<T> {
    receiver: oneshot::Receiver<T>,
}

impl<T> ComputeTask<T> {
    pub fn poll_naive(&mut self) -> Option<T> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(oneshot::TryRecvError::Empty) => None,
            Err(oneshot::TryRecvError::Disconnected) => {
                panic!("Either a completed task was polled again, or the sender was dropped");
            }
        }
    }
}

impl<T> Future for ComputeTask<T> {
    type Output = T;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.get_mut().receiver.poll(cx) {
            std::task::Poll::Ready(Ok(result)) => std::task::Poll::Ready(result),
            std::task::Poll::Ready(Err(_)) => {
                panic!("Either a completed task was polled again, or the sender was dropped");
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

fn available_parallelism() -> usize {
    thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(1)
}

pub fn create_task_pools() {
    let total_threads = available_parallelism();

    create_rayon_pool(total_threads);

    create_async_io_pool(1);
}

fn create_rayon_pool(threads: usize) {
    debug!("Creating rayon thread pool with {} threads", threads);

    rayon_core::ThreadPoolBuilder::new()
        .num_threads(threads)
        // spawn rayon threads using wasm_thread
        .spawn_handler(|thread| -> io::Result<()> {
            let mut b = thread::Builder::new();
            if let Some(name) = thread.name() {
                b = b.name(name.to_owned());
            }
            if let Some(stack_size) = thread.stack_size() {
                b = b.stack_size(stack_size);
            }
            b.spawn(|| thread.run())?;
            Ok(())
        })
        .build_global()
        .expect("Failed to build global rayon thread pool");
}

fn create_async_io_pool(threads: usize) {
    async_io::EXECUTOR
        .set(async_executor::Executor::new())
        .unwrap();

    let executor = async_io::EXECUTOR.get().unwrap();

    for _ in 0..threads {
        thread::Builder::new()
            .name("async-io".to_owned())
            .spawn(|| {
                futures_lite::future::block_on(executor.run(std::future::pending::<()>()));
            })
            .expect("Failed to spawn async-io thread");
    }
}

pub mod compute {
    use super::ComputeTask;

    pub fn spawn<T, F>(f: F) -> ComputeTask<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (sender, receiver) = oneshot::channel();

        rayon_core::spawn(move || {
            // ignore send errors
            let _ = sender.send(f());
        });

        ComputeTask { receiver }
    }

    pub fn spawn_and_forget<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        rayon_core::spawn(f);
    }
}

pub mod async_io {
    use std::sync::OnceLock;

    use super::AsyncTask;

    pub(crate) static EXECUTOR: OnceLock<async_executor::Executor<'static>> = OnceLock::new();

    pub fn spawn<T, F>(f: F) -> AsyncTask<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let task = EXECUTOR
            .get()
            .expect("async-io executor not initialized")
            .spawn(f);

        AsyncTask { task }
    }
}

/// Blocks the current thread on a future.
///
/// # Examples
///
/// ```
/// let val = shin_tasks::block_on(async {
///     1 + 2
/// });
///
/// assert_eq!(val, 3);
/// ```
pub fn block_on<F: Future>(future: F) -> F::Output {
    futures_lite::future::block_on(future)
}
