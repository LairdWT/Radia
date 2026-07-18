use std::future::Future;
use std::pin::pin;
use std::task::{Context, Poll, Waker};
use std::thread;

/// Runs a short native future without adding an async runtime dependency.
pub fn block_on<FutureType: Future>(future: FutureType) -> FutureType::Output {
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let mut future = pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::yield_now(),
        }
    }
}
