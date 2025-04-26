#![deny(missing_docs)]
#![allow(unsafe_code)]
#![allow(unused_variables)]
#![allow(clippy::new_without_default)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::module_inception)]
#![allow(private_bounds)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::await_holding_lock)]
//! [async_task]
mod spawn;
pub use self::spawn::*;
// let mut state_guard = self.runnable.state.lock().unwrap();
//         match &mut *state_guard {
//             // If completed, return the output
//             State::Completed(output) => task::Poll::Ready(output.take()),
//             // If canceled and not running/scheduled, task is fully canceled
//             State::Canceled => task::Poll::Ready(None),
//             // For other states, wait and check again
//             _ => task::Poll::Pending,
//         }
