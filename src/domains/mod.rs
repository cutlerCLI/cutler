// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod collector;
pub mod convert;
pub use collector::{collect, domain_string_to_obj, effective, read_current};
