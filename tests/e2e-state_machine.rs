use libpassport::{state_machine::*, Storage};

use proptest::prelude::*;

use stylus_sdk::alloy_primitives::Address;

fn strat_fillable_sides() -> impl Strategy<Value = (u128, u128, u128, u128)> {
    (100..u128::MAX, 100..u128::MAX).prop_flat_map(|(l_amt, r_amt)| {
        (0..l_amt, 0..r_amt).prop_map(move |(l_ask, r_ask)| (l_amt, r_ask, r_amt, l_ask))
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
        commit_hash in any::<[u8; 64]>(),
        left_balance_hash in any::<[u8; 64]>(),
        right_balance_hash in any::<[u8; 64]>(),
        left_order_hash in any::<[u8; 64]>(),
        right_order_hash in any::<[u8; 64]>(),
        asset_left in any::<Address>(),
        asset_right in any::<Address>(),
        owner_left in any::<Address>(),
        owner_right in any::<Address>(),
        l_ms_ts in any::<u128>(),
        r_ms_ts in any::<u128>(),
        (l_amt, l_ask, r_amt, r_ask) in strat_fillable_sides()
    ) {
        let s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
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
        assert_eq!(asset_left, s.app.order_asset(&l_order));
        assert_eq!(asset_right, s.app.order_asset(&r_order));
        assert_eq!(owner_left, s.app.order_owner(&l_order));
        assert_eq!(owner_right, s.app.order_owner(&r_order));
        assert_eq!(l_amt, s.app.order_from(owner_left, asset_left, &l_order).unwrap());
        assert_eq!(r_amt, s.app.order_from(owner_right, asset_right, &r_order).unwrap());
        let c = Commit::Inline(
            CommitArgs { ms_ts: 0 },
            Box::new(l_order),
            Box::new(r_order),
            commit_hash,
        );
        let left_filled_to_bal = Balance::CommitLeftFilledToBal(Box::new(c.clone()), left_filled_hash);
        let right_filled_to_bal = Balance::CommitRightFilledToBal(Box::new(c.clone()), right_filled_hash);
        assert_eq!(owner_left, s.app.balance_owner(&left_filled_to_bal));
        assert_eq!(asset_right, s.app.balance_asset(&left_filled_to_bal));
        assert_eq!(
            l_ask,
            s.app.balance_amount(owner_right, asset_left, &right_filled_to_bal).unwrap()
        );
        assert_eq!(
            r_ask,
            s.app.balance_amount(owner_left, asset_right, &left_filled_to_bal).unwrap()
        );
        let left_excess_to_order = Order::CommitLeftExcessToOrder(Box::new(c.clone()), left_excess_hash);
        let right_excess_to_order = Order::CommitRightExcessToOrder(Box::new(c.clone()), right_excess_hash);
        assert_eq!(owner_left, s.app.order_owner(&left_excess_to_order));
        assert_eq!(asset_left, s.app.order_asset(&left_excess_to_order));
        assert_eq!(owner_right, s.app.order_owner(&right_excess_to_order));
        assert_eq!(asset_right, s.app.order_asset(&right_excess_to_order));
        assert_eq!(
            l_amt - r_ask,
            s.app.order_from(owner_left, asset_left, &left_excess_to_order).unwrap()
        );
        assert_eq!(
            r_amt - l_ask,
            s.app.order_from(owner_right, asset_right, &right_excess_to_order).unwrap()
        );
        let left_excess_to_order_cancel = Balance::Cancel(
            Box::new(left_excess_to_order),
            left_cancel_hash
        );
        let right_excess_to_order_cancel = Balance::Cancel(
            Box::new(right_excess_to_order),
            right_cancel_hash
        );
        assert_eq!(owner_left, s.app.balance_owner(&left_excess_to_order_cancel));
        assert_eq!(asset_left, s.app.balance_asset(&left_excess_to_order_cancel));
        assert_eq!(owner_right, s.app.balance_owner(&right_excess_to_order_cancel));
        assert_eq!(asset_right, s.app.balance_asset(&right_excess_to_order_cancel));
    }
}
