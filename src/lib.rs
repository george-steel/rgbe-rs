//! A library for loading and handling RGBE-format HDR textures.
//! 
//! Provides types for handling common-exponent floating point texture formats as well as
//! conversions between them and independent floating-point channels.
//! Supports the [RGBE8] format which is storable in Radiance HDR and PNG files,
//! as well as the [RGB9E5] GPU texture format.
//!
//! An intended use case for this ligrary is to store HDR textures as RGBE8 PNG files
//! and convert them to RGB9E5 for the GPU when loading.
//!
//! Radiance HDR images can be converted to RGBE8 PNG files on the command line using
//! the included `hdr2rgbe-png` tool.

mod types;
mod load;

pub use crate::types::*;
pub use crate::load::*;