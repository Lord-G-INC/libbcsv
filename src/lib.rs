// Default mods and exports
pub mod hash;
pub mod types;
pub mod csv;
pub mod convert;
pub type BcsvError = Box<dyn Error>;
pub use binrw::Endian;
pub use binrw;
// Crate only exports
use binrw::prelude::*;
use std::error::Error;
// Feature only mods


#[cfg(all(feature = "c_exports", not(target_arch="wasm32-unknown-unknown")))]
pub mod c_exports;
#[cfg(all(feature = "cxx", not(target_arch="wasm32-unknown-unknown")))]
pub mod cxx_exports;
#[cfg(target_arch="wasm32")]
pub mod wasm_exports;