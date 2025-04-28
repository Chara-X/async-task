use std::{future::Future, mem, pin, sync, task};
/// [async_task::spawn]
pub fn spawn<T>(
    future: impl Future<Output = T> + Send + Sync + 'static,
    schedule: impl Fn(sync::Arc<Runnable<T>>) + Send + Sync + 'static,
) -> Task<T>
where
    T: Send + Sync + 'static,
{
    let runnable = sync::Arc::new(Runnable {
        future: sync::Mutex::new(Box::pin(future)),
        schedule: Box::new(schedule),
        state: sync::Mutex::new(State::Scheduled),
    });
    (runnable.schedule)(runnable.clone());
    Task { runnable }
}
/// [async_task::Runnable]
pub struct Runnable<T>
where
    T: Send + Sync + 'static,
{
    future: sync::Mutex<pin::Pin<Box<dyn Future<Output = T> + Send + Sync>>>,
    schedule: Box<dyn Fn(sync::Arc<Runnable<T>>) + Send + Sync>,
    state: sync::Mutex<State<T>>,
}
impl<T> Runnable<T>
where
    T: Send + Sync + 'static,
{
    /// [async_task::Runnable::run]
    pub fn run(self: sync::Arc<Self>) -> bool {
        let waker = task::Waker::from(self.clone());
        let mut cx = task::Context::from_waker(&waker);
        let mut state = self.state.lock().unwrap();
        match *state {
            State::Completed(_) | State::Canceled => return false,
            _ => *state = State::Running,
        }
        let mut future = self.future.lock().unwrap();
        *state = match future.as_mut().poll(&mut cx) {
            task::Poll::Ready(output) => State::Completed(Some(output)),
            task::Poll::Pending => State::Pending,
        };
        false
    }
}
impl<T> task::Wake for Runnable<T>
where
    T: Send + Sync + 'static,
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
pub struct Task<T>
where
    T: Send + Sync + 'static,
{
    runnable: sync::Arc<Runnable<T>>,
}
impl<T> Task<T>
where
    T: Send + Sync + 'static,
{
    /// [async_task::Task::detach]
    pub fn detach(self) {
        mem::forget(self);
    }
    /// [async_task::Task::cancel]
    pub async fn cancel(self) -> Option<T> {
        let mut state = self.runnable.state.lock().unwrap();
        match *state {
            State::Completed(_) => {}
            _ => *state = State::Canceled,
        }
        drop(state);
        self.await
    }
}
impl<T> Drop for Task<T>
where
    T: Send + Sync + 'static,
{
    fn drop(&mut self) {
        let mut state = self.runnable.state.lock().unwrap();
        match *state {
            State::Completed(_) => {}
            _ => *state = State::Canceled,
        }
    }
}
impl<T> Future for Task<T>
where
    T: Send + Sync + 'static,
{
    type Output = Option<T>;
    fn poll(self: pin::Pin<&mut Self>, _cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
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
