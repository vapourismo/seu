#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use macros::DataType;
#[doc(hidden)]
pub use seu_core as core;
#[doc(hidden)]
pub use seu_macros as macros;
