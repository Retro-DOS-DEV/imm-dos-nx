//! The dos module contains all of the internal structs used by DOS APIs, as
//! well as methods to manipulate the current VM process space.

pub mod devices;
#[cfg(not(test))]
pub mod emulation;
pub mod errors;
pub mod execution;
pub mod files;
pub mod memory;
pub mod registers;
pub mod state;
