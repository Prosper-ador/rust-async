use std::{
    collections::{VecDeque, BinaryHeap},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
    time::{Duration, Instant},
};

// === MiniRuntime ===

pub struct MiniRuntime {
    queue: Arc<TaskQueue>,
}

impl MiniRuntime {
    pub fn new() -> Self {
        MiniRuntime {
            queue: Arc::new(TaskQueue::new()),
        }
    }

    pub fn block_on<F: Future<Output = ()>>(&mut self, fut: F) {
        let queue = self.queue.clone();
        queue.spawn(fut);

        while queue.run_once() {}
    }
}

// === Spawning ===

pub fn spawn<F>(future: F) -> JoinHandle
where
    F: Future<Output = ()> + Send + 'static,
{
    let handle = JoinHandle::new();
    let done = handle.done.clone();
    let fut = Box::pin(async move {
        future.await;
        done.store(true, Ordering::SeqCst);
    });
    TASK_QUEUE.with(|q| q.spawn(fut));
    handle
}

// === JoinHandle ===

#[derive(Clone)]
pub struct JoinHandle {
    done: Arc<AtomicBool>,
}

impl JoinHandle {
    fn new() -> Self {
        Self {
            done: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Future for JoinHandle {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done.load(Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref(); // Cooperative re-check
            Poll::Pending
        }
    }
}

// === TaskQueue ===

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct TaskQueue {
    tasks: Mutex<VecDeque<BoxedFuture>>,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            tasks: Mutex::new(VecDeque::new()),
        }
    }

    fn spawn(&self, fut: BoxedFuture) {
        self.tasks.lock().unwrap().push_back(fut);
    }

    fn run_once(&self) -> bool {
        if let Some(mut fut) = self.tasks.lock().unwrap().pop_front() {
            let waker = dummy_waker();
            let mut cx = Context::from_waker(&waker);

            if let Poll::Pending = fut.as_mut().poll(&mut cx) {
                self.spawn(fut);
            }
            true
        } else {
            thread::sleep(Duration::from_millis(1));
            false
        }
    }
}

// === Thread-local task queue access ===

thread_local! {
    static TASK_QUEUE: Arc<TaskQueue> = Arc::new(TaskQueue::new());
}

// === Dummy waker ===

fn dummy_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    fn dummy_raw_waker() -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }

    static VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

// === sleep ===

pub async fn sleep(duration: Duration) {
    TimerFuture::new(duration).await;
}

struct TimerFuture {
    wake_time: Instant,
}

impl TimerFuture {
    fn new(duration: Duration) -> Self {
        Self {
            wake_time: Instant::now() + duration,
        }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.wake_time {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref(); // retry later
            Poll::Pending
        }
    }
}

// === yield_now ===

pub async fn yield_now() {
    struct YieldNow;

    impl Future for YieldNow {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }

    YieldNow.await;
}

// === join_all! macro ===

#[macro_export]
macro_rules! join_all {
    ($($fut:expr),+ $(,)?) => {
        futures::future::join_all(vec![$($fut),+]).await;
    };
}

// === mini_rt! macro ===

#[macro_export]
macro_rules! mini_rt {
    (async fn $name:ident() $body:block) => {
        fn main() {
            let mut rt = $crate::MiniRuntime::new();
            rt.block_on(async { $body });
        }
    };
}
