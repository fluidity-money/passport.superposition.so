#[cfg(not(target_arch = "wasm32"))]
use stylus_sdk::alloy_primitives::{Address, FixedBytes, U256};

#[cfg(not(target_arch = "wasm32"))]
use proptest::{prelude::Rng, strategy::Strategy};

#[cfg(not(target_arch = "wasm32"))]
#[derive(PartialEq)]
pub enum Uintsize {
    Small,
    Medium,
    Large,
}

/// Simple strategy that generates values up to a million.
#[cfg(not(target_arch = "wasm32"))]
pub fn strat_tiny_u256() -> impl proptest::prelude::Strategy<Value = U256> {
    (0..1_000_000).prop_map(|x| U256::from(x))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_fixed_bytes_sizeable<const N: usize>(
    u: Uintsize,
) -> impl proptest::prelude::Strategy<Value = FixedBytes<N>> {
    // Create a slice of fixed bytes, with a preference for the lower side, a
    // la how I recall seeing Parity's Ethereum client do it. This has a 33%
    // chance of filling out a third of the lower bits, which, in our
    // interpretation, is decoded as big endian in the next function, so
    // the right side, a 33% chance of two thirds, and a 33% chance of
    // everything is potentially filled out.
    (0..3).prop_perturb(move |s, mut rng| {
        let mut x: [u8; N] = [0u8; N];
        let q = N / 3;
        if s == 2 && u == Uintsize::Large {
            for i in q * 2..N {
                x[N - i - 1] = rng.gen();
            }
        }
        if s >= 1 && u != Uintsize::Small {
            for i in q..q * 2 {
                x[N - i - 1] = rng.gen();
            }
        }
        for i in 0..q {
            x[N - i - 1] = rng.gen();
        }
        FixedBytes::<N>::from(x)
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_u256(s: Uintsize) -> impl proptest::prelude::Strategy<Value = U256> {
    strat_fixed_bytes_sizeable::<32>(s).prop_map(|x| U256::from_be_bytes(x.into()))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_small_u256() -> impl proptest::prelude::Strategy<Value = U256> {
    strat_u256(Uintsize::Small)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_medium_u256() -> impl proptest::prelude::Strategy<Value = U256> {
    strat_u256(Uintsize::Medium)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_large_u256() -> impl proptest::prelude::Strategy<Value = U256> {
    strat_u256(Uintsize::Large)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_fixed_bytes<const N: usize>() -> impl proptest::prelude::Strategy<Value = FixedBytes<N>>
{
    strat_fixed_bytes_sizeable::<N>(Uintsize::Large)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_address_not_empty() -> impl proptest::prelude::Strategy<Value = Address> {
    ([
        1..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
        0..u8::MAX,
    ])
    .prop_map(Address::new)
}
