// Tests that go the full way of asking questions about the applicative
// form before comparing the outcome in the state machine to the final
// outcome.

use libpassport::{
    immutables::solver_key_offline, network::Network, solver_context::*,
    utils::*, user_context::*, Storage
};

mod experimentation;

use experimentation::*;

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

use stylus_sdk::alloy_primitives::{FixedBytes};

proptest! {
    #[test]
    fn test_apply(
        signer_priv_key in any::<[u8; 32]>(),
        signer_addr in strat_address_not_empty(),
        e: Entry
    ) {
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
        s.app.ed25519_keys.setter(0).set(FixedBytes::from_slice(signer_pub.as_bytes()));
        s.app.ed25519_owners.setter(0).set(signer_addr);
        apply_balances(&mut s, starting_amts(&e));
        s.app.apply(
            s.app.validate(Network::OFFLINE, &user_ctx.accounts, converted)
                .unwrap()
        )
            .unwrap();
    }
}
