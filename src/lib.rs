// Default mods and exports
pub mod hash;
pub mod types;
pub mod csv_parse;
pub mod string_table;
pub type BcsvError = Box<dyn Error>;
pub use binrw::Endian;
pub use binrw;
// Crate only exports
use binrw::prelude::*;
use std::error::Error;
// Feature only mods

#[cfg(feature = "c_exports")]
pub mod c_exports;
#[cfg(feature = "cxx")]
pub mod cxx_exports;