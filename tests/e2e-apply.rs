// Tests that go the full way of asking questions about the applicative
// form before comparing the outcome in the state machine to the final
// outcome.

use libpassport::{
    applicative::{ArgsBalance, ArgsOrder},
    immutables::solver_key_offline,
    network::Network,
    solver_context::*,
    user_context::*,
    Storage,
};

mod experimentation;

use experimentation::*;

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

use stylus_sdk::{alloy_primitives::FixedBytes, prelude::HostAccess};

proptest! {
    #[test]
    #[ignore]
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
        s.app.ed25519_keys.setter(0).set(FixedBytes::from_slice(signer_pub.as_bytes()));
        s.app.ed25519_owners.setter(0).set(msg_sender);
        apply_balances(&mut s, starting_amts(&e)).unwrap();
        s.app.apply(
            s.app.validate(Network::OFFLINE, &user_ctx.accounts, converted)
                .unwrap()
        )
            .unwrap();
    }
}

#[test]
fn test_apply_2() {
    let signer_priv_key = [0u8; 32];
    let e = Entry::Order(TestOrder::Order(Box::new(TestOrderInside {
        from: Box::new(TestBalance::Balance(TestBalanceInside {
            args: ArgsBalance {
                asset: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                chain: 0,
                amount: 1000,
                ms_timestamp: 54605753107711057641113418769989,
            },
        })),
        args: ArgsOrder {
            from_amt: 100,
            desired_asset: [
                65, 110, 126, 3, 66, 181, 50, 169, 220, 157, 116, 201, 152, 224, 231, 144, 239,
                100, 246, 3,
            ],
            desired_chain: 126508738668270503307039462831516350703,
            desired_amt: 88720005195802133222596758088858595407,
        },
    })));
    let signer_priv = SigningKey::from_bytes(&signer_priv_key);
    let signer_pub = signer_priv.verifying_key();
    let solver_ctx = SolverContext::new(solver_key_offline());
    // Implicitly registers the solver key.
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
    let mut s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
    let msg_sender = s.vm().msg_sender();
    s.app
        .ed25519_keys
        .setter(0)
        .set(FixedBytes::from_slice(signer_pub.as_bytes()));
    s.app.ed25519_owners.setter(0).set(msg_sender);
    apply_balances(&mut s, starting_amts(&e)).unwrap();
    s.app
        .apply(
            s.app
                .validate(Network::OFFLINE, &user_ctx.accounts, converted)
                .unwrap(),
        )
        .unwrap();
}
