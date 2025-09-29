use libpassport::{
    applicative::Applicative, immutables::solver_key_offline, solver_context::SolverContext,
    user_context::UserContext,
};

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

mod experimentation;

use experimentation::{convert, Entry};

use borsh::{BorshDeserialize, ser::BorshSerialize};

proptest! {
    #[test]
    fn test_encode_decode(signer_priv_key in any::<[u8; 32]>(), e in any::<Entry>()) {
        let signer_priv = SigningKey::from_bytes(&signer_priv_key);
        let solver_ctx = SolverContext::new(solver_key_offline());
        let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
        let mut buf = Vec::new();
        let c = convert(&user_ctx, &solver_ctx, &e).unwrap();
        c.serialize(&mut buf).unwrap();
        assert_eq!(c, Applicative::try_from_slice(&mut buf).unwrap());
    }
}
