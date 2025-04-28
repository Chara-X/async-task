#![deny(missing_docs)]
#![allow(unsafe_code)]
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
