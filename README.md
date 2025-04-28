# async-task

## Example

```rust
use std::{
    sync::{self, mpsc},
    thread, time,
};
fn main() {
    let (tx, rx) = mpsc::channel::<sync::Arc<async_task::Runnable<()>>>();
    thread::spawn(move || {
        for runnable in rx {
            runnable.run();
        }
    });
    let sender = tx.clone();
    let task = async_task::spawn(
        async {
            println!("Task 1 running");
            another_task().await;
        },
        move |runnable| {
            sender.send(runnable).unwrap();
        },
    );
    task.detach();
    let sender = tx.clone();
    let task = async_task::spawn(
        async {
            println!("Task 2 running");
            another_task().await;
        },
        move |runnable| {
            sender.send(runnable).unwrap();
        },
    );
    task.detach();
    thread::sleep(time::Duration::from_secs(1));
    // Output:
    // Task 1 running
    // Just another task
    // Task 2 running
    // Just another task
}
async fn another_task() {
    println!("Just another task");
}
```
