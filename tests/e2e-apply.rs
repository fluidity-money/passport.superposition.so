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

#[test]
fn test_simple_apply() {
    let signer_priv_key = [
        38, 37, 197, 255, 30, 234, 252, 184, 30, 168, 152, 209, 36, 146, 88, 141, 213, 220, 61,
        191, 121, 200, 145, 223, 34, 75, 129, 113, 104, 12, 124, 202,
    ];
    storage_host::storage_clear();
    call_eip20_extras::clear();
    set_msg_sender([1u8; 20]);
    let e = Entry::MakeOrder(TestBalance::Cancel(Box::new(TestOrder::Order(Box::new(
        TestOrderInside {
            from: Box::new(TestBalance::Balance(TestBalanceInside {
                args: ArgsBalance {
                    asset: Asset([1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]),
                    chain: 7504632268041403195,
                    amount: U128(54491797398298066872776494644143039556),
                    ms_timestamp: U128(265299708992243408992440341416264526405),
                },
            })),
            args: ArgsOrder {
                from_amt: U128(39535890710855483817640651244670823763),
                desired_asset: Asset([
                    207, 247, 13, 167, 215, 134, 71, 121, 23, 104, 216, 239, 235, 40, 198, 249,
                    139, 128, 244, 18,
                ]),
                desired_chain: U128(230100668142313416539596414952344424456),
                desired_amt: U128(327435873575427839506168674194457076493),
            },
        },
    )))));
    let signer_priv = SigningKey::from_bytes(&signer_priv_key);
    let signer_pub = signer_priv.verifying_key();
    let solver_ctx = SolverContext::new(solver_key_offline());
    // Implicitly registers the solver key.
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
    storage::ed25519_keys::set(&U::ZERO, signer_pub.as_bytes().into());
    storage::ed25519_owners::set(&U::ZERO, &msg_sender().into());
    apply_balances(starting_amts(&e)).unwrap();
    apply(validate(&pick_solver_key(Network::CUSTOM), &vec![0], &converted).unwrap()).unwrap();
}

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
