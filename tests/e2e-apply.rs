// Tests that go the full way of asking questions about the applicative
// form before comparing the outcome in the state machine to the final
// outcome.

use libpassport::{
    applicative::*,
    apply::apply,
    call_eip20_extras,
    conversion::validate,
    immutables::{pick_solver_key, solver_key_offline},
    network::Network,
    solver_context::*,
    storage::{self},
    user_context::*,
};

use bobcat_sdk::{
    entry::{host::set_msg_sender, msg_sender},
    maths::U,
    storage::host as storage_host,
};

mod experimentation;

use experimentation::*;

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

proptest! {
    #[test]
    fn test_apply(signer_priv_key in any::<[u8; 32]>(), e in any::<Entry>()) {
        storage_host::storage_clear();
        call_eip20_extras::clear();
        set_msg_sender([1u8; 20]);
        let signer_priv = SigningKey::from_bytes(&signer_priv_key);
        let signer_pub = signer_priv.verifying_key();
        let solver_ctx = SolverContext::new(solver_key_offline());
        // Implicitly registers the solver key.
        let user_ctx = UserContext::new_from_bytes(
            signer_priv.as_bytes(),
            0
        );
        let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
        storage::ed25519_keys::set(&U::ZERO, signer_pub.as_bytes().into());
        storage::ed25519_owners::set(&U::ZERO, &msg_sender().into());
        apply_balances(starting_amts(&e)).unwrap();
        apply(
            validate(&pick_solver_key(Network::CUSTOM), &vec![0], &converted)
                .unwrap()
        )
            .unwrap();
    }
}
