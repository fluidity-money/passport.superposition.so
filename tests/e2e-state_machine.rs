use libpassport::{
    apply::{
        balance_amount, balance_asset, balance_owner, commit_left_amount_filled, commit_left_asset,
        commit_left_owner, commit_right_amount_filled, commit_right_asset, commit_right_owner,
        order_asset, order_from_amt, order_owner, withdraw_asset, withdraw_balance, withdraw_owner,
    },
    call_eip20_extras,
    state_machine::*,
};

use bobcat_sdk::storage::storage_host;

use proptest::prelude::*;

fn strat_fillable_sides() -> impl Strategy<Value = (u128, u128, u128, u128)> {
    (100..u128::MAX, 100..u128::MAX).prop_flat_map(|(l_amt, r_amt)| {
        (0..l_amt, 0..r_amt).prop_map(move |(r_ask, l_ask)| (l_amt, l_ask, r_amt, r_ask))
    })
}

proptest! {
    #[test]
    fn test_filling_okay(
        left_filled_hash in any::<[u8; 64]>(),
        right_filled_hash in any::<[u8; 64]>(),
        left_excess_hash in any::<[u8; 64]>(),
        right_excess_hash in any::<[u8; 64]>(),
        left_cancel_hash in any::<[u8; 64]>(),
        right_cancel_hash in any::<[u8; 64]>(),
        left_withdraw_filled_hash in any::<[u8; 64]>(),
        right_withdraw_filled_hash in any::<[u8; 64]>(),
        left_withdraw_excess_hash in any::<[u8; 64]>(),
        right_withdraw_excess_hash in any::<[u8; 64]>(),
        commit_hash in any::<[u8; 64]>(),
        left_balance_hash in any::<[u8; 64]>(),
        right_balance_hash in any::<[u8; 64]>(),
        left_order_hash in any::<[u8; 64]>(),
        right_order_hash in any::<[u8; 64]>(),
        asset_left in any::<[u8; 20]>(),
        asset_right in any::<[u8; 20]>(),
        owner_left in any::<[u8; 20]>(),
        owner_right in any::<[u8; 20]>(),
        l_ms_ts in any::<u128>(),
        r_ms_ts in any::<u128>(),
        (l_amt, l_ask, r_amt, r_ask) in strat_fillable_sides()
    ) {
        storage_host::storage_clear();
        call_eip20_extras::clear();
        // Create left order to be competely filled
        let l_order = Order::Inline(
            OrderArgs {
                desired_asset: asset_right,
                desired_amt: l_ask,
                from_amt: l_amt,
                max_pol_fee: 0,
                ord_partial_fill_okay: false,
            },
            Box::new(Balance::Inline(
                BalanceArgs {
                    ms_ts: l_ms_ts,
                    owner: owner_left,
                    asset: asset_left,
                    amt: l_amt,
                },
                left_balance_hash,
            )),
            left_order_hash,
        );
        // Create right order to be completely filled
        let r_order = Order::Inline(
            OrderArgs {
                desired_asset: asset_left,
                desired_amt: r_ask,
                from_amt: r_amt,
                max_pol_fee: 0,
                ord_partial_fill_okay: false,
            },
            Box::new(Balance::Inline(
                BalanceArgs {
                    ms_ts: r_ms_ts,
                    owner: owner_right,
                    asset: asset_right,
                    amt: r_amt,
                },
                right_balance_hash,
            )),
            right_order_hash,
        );
        // Assert owner/asset/amount_from of both orders is correct
        assert_eq!(asset_left, order_asset(&l_order));
        assert_eq!(asset_right, order_asset(&r_order));
        assert_eq!(owner_left, order_owner(&l_order));
        assert_eq!(owner_right, order_owner(&r_order));
        assert_eq!(
            l_amt,
            order_from_amt(
                owner_left,
                asset_left,
                &l_order
            )
            .unwrap()
        );
        assert_eq!(
            r_amt,
           order_from_amt(
                owner_right,
                asset_right,
                &r_order
            )
            .unwrap()
        );
        // Create commit and fills of each side
        let c = Commit::Inline(
            CommitArgs { ms_ts: 0 },
            Box::new(l_order),
            Box::new(r_order),
            commit_hash,
        );
        let left_filled_to_bal = Balance::CommitLeftFilledToBal(Box::new(c.clone()), left_filled_hash);
        let right_filled_to_bal =
            Balance::CommitRightFilledToBal(Box::new(c.clone()), right_filled_hash);
        // Left filled balance is the right asset, owned by the left user
        assert_eq!(owner_left, balance_owner(&left_filled_to_bal));
        assert_eq!(asset_right, balance_asset(&left_filled_to_bal));
        // Right amount filled is owned by the right, contains left asset, is the amount the right side desired
        assert_eq!(
            r_ask,
            balance_amount(
                    owner_right,
                    asset_left,
                    &right_filled_to_bal
                )
                .unwrap()
        );
        // Left amount filled is owned by the left, contains right asset, is the amount the left side desired
        assert_eq!(
            l_ask,
            balance_amount(
                    owner_left,
                    asset_right,
                    &left_filled_to_bal
                )
                .unwrap()
        );
        let left_excess_to_order =
            Order::CommitLeftExcessToOrder(Box::new(c.clone()), left_excess_hash);
        let right_excess_to_order =
            Order::CommitRightExcessToOrder(Box::new(c.clone()), right_excess_hash);
        assert_eq!(owner_left, order_owner(&left_excess_to_order));
        assert_eq!(asset_left, order_asset(&left_excess_to_order));
        assert_eq!(owner_right, order_owner(&right_excess_to_order));
        assert_eq!(asset_right, order_asset(&right_excess_to_order));
        // The leftover amount of the left asset is made into a new order, which should have amount "total left asset - amount of left asset desired by right user"
        assert_eq!(
            l_amt - r_ask,
            order_from_amt(
                    owner_left,
                    asset_left,
                    &left_excess_to_order
                )
                .unwrap()
        );
        assert_eq!(
            r_amt - l_ask,
            order_from_amt(
                    owner_right,
                    asset_right,
                    &right_excess_to_order
                )
                .unwrap()
        );
        let left_excess_to_order_cancel =
            Balance::Cancel(Box::new(left_excess_to_order), left_cancel_hash);
        let right_excess_to_order_cancel =
            Balance::Cancel(Box::new(right_excess_to_order), right_cancel_hash);
        assert_eq!(
            owner_left,
            balance_owner(&left_excess_to_order_cancel)
        );
        assert_eq!(
            asset_left,
            balance_asset(&left_excess_to_order_cancel)
        );
        assert_eq!(
            owner_right,
            balance_owner(&right_excess_to_order_cancel)
        );
        assert_eq!(
            asset_right,
            balance_asset(&right_excess_to_order_cancel)
        );
        // Withdraw excess
        let left_excess_withdraw = Withdraw::Inline(Box::new(left_excess_to_order_cancel), left_withdraw_excess_hash);
        let right_excess_withdraw = Withdraw::Inline(Box::new(right_excess_to_order_cancel), right_withdraw_excess_hash);
        assert_eq!(
            owner_left,
            withdraw_owner(&left_excess_withdraw)
        );
        assert_eq!(
            asset_left,
            withdraw_asset(&left_excess_withdraw)
        );
        assert_eq!(
            l_amt - r_ask,
            withdraw_balance(owner_left, asset_left, &left_excess_withdraw).unwrap()
        );
        assert_eq!(
            owner_right,
            withdraw_owner(&right_excess_withdraw)
        );
        assert_eq!(
            asset_right,
            withdraw_asset(&right_excess_withdraw)
        );
        assert_eq!(
            r_amt - l_ask,
            withdraw_balance(owner_right, asset_right, &right_excess_withdraw).unwrap()
        );
        // Then withdraw both sides filled
        let left_filled_withdraw = Withdraw::Inline(Box::new(left_filled_to_bal), left_withdraw_filled_hash);
        let right_filled_withdraw = Withdraw::Inline(Box::new(right_filled_to_bal), right_withdraw_filled_hash);
        assert_eq!(
            owner_left,
            withdraw_owner(&left_filled_withdraw)
        );
        assert_eq!(
            asset_right,
            withdraw_asset(&left_filled_withdraw)
        );
        assert_eq!(
            l_ask,
            withdraw_balance(owner_left, asset_right, &left_filled_withdraw).unwrap()
        );
        assert_eq!(
            owner_right,
            withdraw_owner(&right_filled_withdraw)
        );
        assert_eq!(
            asset_left,
            withdraw_asset(&right_filled_withdraw)
        );
        assert_eq!(
            r_ask,
            withdraw_balance(owner_right, asset_left, &right_filled_withdraw).unwrap()
        );
    }
}
