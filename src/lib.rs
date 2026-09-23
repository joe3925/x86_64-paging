#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod address;
pub mod attrs;
pub mod descriptor;
pub mod mapper;
pub mod regime;
pub mod table;
pub mod translation;

pub use address::{Granule4KiB, canonicalize, from_canonical};
pub use attrs::{LeafAttrs, MemoryType, PageTableFlags, TableAttrs};
pub use descriptor::{Amd64, Amd64Layout};
pub use paging;
pub use regime::{LongMode, LongMode4Level, LongMode5Level};

pub type InputAddress = u64;
pub type OutputAddress = u64;
