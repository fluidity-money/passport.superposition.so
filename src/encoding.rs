
use stylus_sdk::alloy_primitives::*;

use borsh::{BorshDeserialize, BorshSerialize};

macro_rules! borsh_encoding {
    ( $( ($type:ty, $size:expr, $from_fn:path, $to_fn:expr) ),* $(,)? ) => {
        $(
            paste::paste! {
                pub struct [<B$type>] {
                    pub x: $type,
                }

                impl BorshSerialize for [<B$type>] {
                    fn serialize<W: borsh::io::Write>(&self, w: &mut W) -> Result<(), borsh::io::Error> {
                        w.write_all(&(($to_fn)(self.x)))
                    }
                }

                impl BorshDeserialize for [<B$type>] {
                    fn deserialize_reader<R: borsh::io::Read>(r: &mut R) -> Result<Self, borsh::io::Error> {
                        let mut x = [0u8; $size];
                        r.read(&mut x)?;
                        Ok(Self {
                            x: $from_fn(x),
                        })
                    }
                }

                impl From<$type> for [<B$type>] {
                    fn from(x: $type) -> Self {
                        Self { x }
                    }
                }

                impl Into<$type> for [<B$type>] {
                    fn into(self) -> $type {
                        self.x
                    }
                }
            }
        )*
    };
}

borsh_encoding! {
    (U256, 32, U256::from_le_bytes, |x: U256| x.to_le_bytes_trimmed_vec()),
    (Address, 20, Address::from, Address::into_array),
}
