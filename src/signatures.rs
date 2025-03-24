use crate::*;

use sha2::{Digest, Sha256};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

pub fn validate_req(verifier: &[u8], req: &MatchReq, sig: &EdSig) -> bool {
    true
}

/// Hash a match request, by concatenating its fields, then hashing it using sha2.
pub fn hash_match_req(r: &MatchReq) -> [u8; 32] {
    let mut b = [0u8; 20 + 32 + 20 + 32 + 32 + 8];
    b[0..20].copy_from_slice(&r.spend_token.x.into_array());
    b[20..52].copy_from_slice(&r.spend_amt.x.to_be_bytes::<32>());
    b[52..72].copy_from_slice(&r.goal_token.x.into_array());
    b[72..104].copy_from_slice(&r.goal_amt.x.to_be_bytes::<32>());
    b[104..136].copy_from_slice(&r.nonce.x.to_be_bytes::<32>());
    b[136..144].copy_from_slice(&r.deadline.to_be_bytes());
    let mut h = Sha256::new();
    h.update(&b);
    h.finalize().as_slice().try_into().unwrap()
}

/// Create a signature using ed25519, using the browser context API.
pub fn create_sig(signing_key: [u8; 32], req: &MatchReq) -> EdSig {
    let s = SigningKey::from_bytes(&[1u8; 32]);
    let h = hash_match_req(req);
    let s = s.sign(&h);
    EdSig {
        r: *s.r_bytes(),
        s: *s.s_bytes(),
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod test {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_hash_match_req_no_crash(r in strat_match_req()) {
            hash_match_req(&r);
        }
    }
}
