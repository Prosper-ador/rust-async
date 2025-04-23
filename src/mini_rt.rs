use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
    time::{Duration, Instant},
    cell::RefCell,
};

// === MiniRuntime ===

pub struct MiniRuntime {
    queue: Arc<TaskQueue>,
}

impl MiniRuntime {
    pub fn new() -> Self {
        let queue = Arc::new(TaskQueue::new());
        TASK_QUEUE.with(|q| *q.borrow_mut() = queue.clone());
        MiniRuntime { queue }
    }

    pub fn block_on<F: Future<Output = ()> + Send + 'static>(&mut self, fut: F) {
        self.queue.spawn(Box::pin(fut));
        while self.queue.run_once() {}
    }
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
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// === spawn ===

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

    TASK_QUEUE.with(|q| {
        q.borrow().spawn(fut);
    });

    handle
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

// === Thread-local access to task queue ===

thread_local! {
    static TASK_QUEUE: RefCell<Arc<TaskQueue>> = RefCell::new(Arc::new(TaskQueue::new()));
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
            cx.waker().wake_by_ref();
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

// === join_all! ===

#[macro_export]
macro_rules! join_all {
    ($($fut:expr),+ $(,)?) => {
        {
            let mut handles = vec![$($fut),+];
            for h in handles.iter_mut() {
                h.await;
            }
        }
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
