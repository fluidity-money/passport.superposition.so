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
    entry::{impls::set_msg_sender, msg_sender},
    maths::U,
    storage::storage_host,
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

// Given a Commit, withdraw CommitLeftFilledToBalance and
// CommitRightFilledToBalance
#[test]
fn test_apply_withdraw_both_balances() {
    let signer_priv_key = [0; 32];
    let asset_left = Asset([1; 20]);
    let asset_right = Asset([2; 20]);
    let chain: u64 = 1;
    let desired_chain = U128(chain.into());
    let ms_timestamp = U128(1);
    let amount_left = U128(15);
    let amount_right = U128(10);
    // Trade 15 of asset_left for 10 of asset_right
    let test_commit = TestCommit::Commit(Box::new(TestCommitInside {
        args: ArgsCommit {
            ms_timestamp: ms_timestamp.clone(),
        },
        left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
            from: Box::new(TestBalance::Balance(TestBalanceInside {
                args: ArgsBalance {
                    asset: asset_left.clone(),
                    chain,
                    amount: amount_left.clone(),
                    ms_timestamp: ms_timestamp.clone(),
                },
            })),
            args: ArgsOrder {
                from_amt: amount_left.clone(),
                desired_asset: asset_right.clone(),
                desired_chain: desired_chain.clone(),
                desired_amt: amount_right.clone(),
            },
        }))),
        right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
            from: Box::new(TestBalance::Balance(TestBalanceInside {
                args: ArgsBalance {
                    asset: asset_right.clone(),
                    chain,
                    amount: amount_right.clone(),
                    ms_timestamp: ms_timestamp.clone(),
                },
            })),
            args: ArgsOrder {
                from_amt: amount_right.clone(),
                desired_asset: asset_left.clone(),
                desired_chain: desired_chain.clone(),
                desired_amt: amount_left.clone(),
            },
        }))),
    }));
    let e_left = Entry::Withdraw(TestBalance::CommitLeftFilledToBalance(Box::new(
        test_commit.clone(),
    )));
    let e_right = Entry::Withdraw(TestBalance::CommitRightFilledToBalance(Box::new(
        test_commit.clone(),
    )));
    storage_host::storage_clear();
    call_eip20_extras::clear();
    set_msg_sender([1u8; 20]);
    let signer_priv = SigningKey::from_bytes(&signer_priv_key);
    let signer_pub = signer_priv.verifying_key();
    let solver_ctx = SolverContext::new(solver_key_offline());
    // Implicitly registers the solver key.
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    // Setup happens once, since we want to maintain the balances when withdrawing the right side
    storage::ed25519_keys::set(&U::ZERO, signer_pub.as_bytes().into());
    storage::ed25519_owners::set(&U::ZERO, &msg_sender().into());
    apply_balances(starting_amts(&e_left)).unwrap();

    // Withdraw the left balance
    let converted_left = convert(&user_ctx, &solver_ctx, &e_left).unwrap();
    apply(validate(&pick_solver_key(Network::CUSTOM), &vec![0], &converted_left).unwrap()).unwrap();

    // Withdraw the right balance
    let converted_right = convert(&user_ctx, &solver_ctx, &e_right).unwrap();
    apply(
        validate(
            &pick_solver_key(Network::CUSTOM),
            &vec![0],
            &converted_right,
        )
        .unwrap(),
    )
    .unwrap();
}
