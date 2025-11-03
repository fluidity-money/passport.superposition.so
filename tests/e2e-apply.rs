// Tests that go the full way of asking questions about the applicative
// form before comparing the outcome in the state machine to the final
// outcome.

use libpassport::{
    immutables::{pick_solver_key, solver_key_offline},
    network::Network,
    solver_context::*,
    user_context::*,
    Storage,
};

mod experimentation;

use experimentation::*;

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

proptest! {
    #[test]
    fn test_apply(signer_priv_key in any::<[u8; 32]>(), e in any::<Entry>()) {
        let signer_priv = SigningKey::from_bytes(&signer_priv_key);
        let signer_pub = signer_priv.verifying_key();
        let solver_ctx = SolverContext::new(solver_key_offline());
        // Implicitly registers the solver key.
        let user_ctx = UserContext::new_from_bytes(
            signer_priv.as_bytes(),
            0
        );
        let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
        let mut s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
        let msg_sender = s.vm().msg_sender();
        s.app.validation.ed25519_keys.setter(0).set(FixedBytes::from_slice(signer_pub.as_bytes()));
        s.app.validation.ed25519_owners.setter(0).set(msg_sender);
        apply_balances(&mut s, starting_amts(&e)).unwrap();
        s.app.apply(
            s.app.validation.validate(&pick_solver_key(Network::CUSTOM), &vec![0], &converted)
                .unwrap()
        )
            .unwrap();
    }
}
