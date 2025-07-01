
use crate::{applicative::*, error::*};

use stylus_sdk::crypto::keccak;

// Validate the Balance by allocating in the stack a buffer for the
// conversion of the CreateBalance argument to a local encoding. Hash
// that then sign it, then derive the signature based on the arguments
// given.
pub fn validate_balance(sig: &[u8; 64], ap: &ArgsBalance) -> Result<bool, Error> {
}
