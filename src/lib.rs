#![cfg_attr(doc, feature(doc_cfg))]

//! A library for reading and writing the [BCSV File Format](https://www.lumasworkshop.com/wiki/BCSV_(File_format)).
//! For most people, the [`types::BCSV`] struct is what you'll be using.
//! For some, the [`hash`] and [`csv_parse`] modules may have more.
//! Either way, this library exists as a way to better understand this semi-obscure Nintendo format.

// Default mods and exports
/// The hashing functions used by Nintendo.
pub mod hash;
/// The inner types of the BCSV format.
pub mod types;
/// The module to parse csv files and convert them to BCSV.
pub mod csv_parse;
/// The module to handle the BCSV string table.
pub mod string_table;
/// A module to handle holding BCSV fields and their info.
pub mod field_holder;
pub type BcsvError = Box<dyn Error>;
pub use binrw::Endian;
pub use binrw;
// Crate only exports
use binrw::prelude::*;
use std::error::Error;
// Feature only mods

#[cfg(feature = "c_exports")]
#[cfg_attr(doc, doc(cfg(c_exports)))]
/// C exported functions of the library.
pub mod c_exports;
#[cfg(feature = "cxx")]
#[cfg_attr(doc, doc(cfg(cxx)))]
/// C++ exported functions of the library.
pub mod cxx_exports;
/// [`serde::Serialize`] and [`serde::Deserialize`] implentaions for the crate.
#[cfg(feature = "serde")]
#[cfg_attr(doc, doc(cfg(serde)))]
pub mod serde_impls;