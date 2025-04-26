use std::{future::Future, mem, pin, sync, task};
/// [async_task::spawn]
pub fn spawn<F, S>(future: F, schedule: S) -> Task<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    let runnable = sync::Arc::new(Runnable {
        future: sync::Mutex::new(future),
        schedule,
        state: sync::Mutex::new(State::Scheduled),
    });
    (runnable.schedule)(runnable.clone());
    Task { runnable }
}
/// [async_task::Runnable]
pub struct Runnable<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    future: sync::Mutex<F>,
    schedule: S,
    state: sync::Mutex<State<F::Output>>,
}
impl<F, S> Runnable<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    /// [async_task::Runnable::run]
    pub fn run(this: sync::Arc<Runnable<F, S>>) -> bool {
        let waker = task::Waker::from(this.clone());
        let mut cx = task::Context::from_waker(&waker);
        let mut state = this.state.lock().unwrap();
        match *state {
            State::Completed(_) | State::Canceled => return false,
            _ => *state = State::Running,
        }
        let mut future = this.future.lock().unwrap();
        *state = match unsafe { pin::Pin::new_unchecked(&mut *future) }.poll(&mut cx) {
            task::Poll::Ready(output) => State::Completed(Some(output)),
            task::Poll::Pending => State::Pending,
        };
        false
    }
}
impl<F, S> task::Wake for Runnable<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    fn wake(self: sync::Arc<Self>) {
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Pending => {
                (self.schedule)(self.clone());
                *state = State::Scheduled;
            }
            State::Running => {
                *state = State::Scheduled;
            }
            _ => {}
        }
    }
}
/// [async_task::Task]
pub struct Task<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    runnable: sync::Arc<Runnable<F, S>>,
}
impl<F, S> Task<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    /// [async_task::Task::detach]
    pub fn detach(self) {
        mem::forget(self);
    }
    /// [async_task::Task::cancel]
    pub async fn cancel(self) -> Option<F::Output> {
        let mut state = self.runnable.state.lock().unwrap();
        match *state {
            State::Completed(_) => {}
            _ => *state = State::Canceled,
        }
        drop(state);
        self.await
    }
}
impl<F, S> Drop for Task<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    fn drop(&mut self) {
        let mut state = self.runnable.state.lock().unwrap();
        match *state {
            State::Completed(_) => {}
            _ => *state = State::Canceled,
        }
    }
}
impl<F, S> Future for Task<F, S>
where
    F: Future + Send + Sync + 'static,
    F::Output: Send + Sync + 'static,
    S: Fn(sync::Arc<Runnable<F, S>>) + Send + Sync + 'static,
{
    type Output = Option<F::Output>;
    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        let waker = task::Waker::from(self.runnable.clone());
        let mut state = self.runnable.state.lock().unwrap();
        match &mut *state {
            State::Completed(output) => task::Poll::Ready(output.take()),
            State::Canceled => task::Poll::Ready(None),
            _ => task::Poll::Pending,
        }
    }
}
enum State<T> {
    Scheduled,
    Running,
    Pending,
    Completed(Option<T>),
    Canceled,
}
