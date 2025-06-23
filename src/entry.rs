use alloc::vec::Vec;

use stylus_sdk::{alloy_primitives::*, prelude::*};

pub use crate::storage::*;

use crate::*;

impl StoragePassport {
    pub fn dummy(&self) -> R {
        NOTHING
    }
}
