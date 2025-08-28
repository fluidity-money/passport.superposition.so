open Passport
open Applicative
open QCheck2.Gen

let gen =
  let gen_positive_int =
    map (( + ) 1) (int_bound (Int32.to_int Int32.max_int))
  in
  let gen_timestamp = map (fun x -> abs x) int in
  let gen_asset = string_size ~gen:(char_range 'A' 'Z') (int_range 3 8) in
  let gen_owner = string_size ~gen:(char_range 'A' 'Z') (int_range 3 8) in

  let rec gen_asset_different_from asset =
    let* a = gen_asset in
    if a = asset then gen_asset_different_from asset else return a
  in

  let rec gen_balance_args () =
    let* bal_ms_ts = gen_timestamp in
    let* bal_asset = gen_asset in
    let* bal_owner = gen_owner in
    let* bal_amt = gen_positive_int in
    return { bal_ms_ts; bal_asset; bal_owner; bal_amt }
  and gen_balance () =
    let* args = gen_balance_args () in
    return (Balance_inline (`Bal_inline args))
  and gen_balance_with_asset asset =
    let* bal_ms_ts = gen_timestamp in
    let* bal_owner = gen_owner in
    let* bal_amt = gen_positive_int in
    return
      (Balance_inline
         (`Bal_inline { bal_ms_ts; bal_asset = asset; bal_owner; bal_amt }))
  and gen_balance_with_asset_different_from asset =
    let* a = gen_asset_different_from asset in
    gen_balance_with_asset a
  and gen_order_from_balance balance =
    let bal_asset = Apply.bal_asset balance in
    let* ord_desired_asset = gen_asset_different_from bal_asset in
    let* ord_desired_amt = gen_positive_int in
    let* ord_max_pol_fee = gen_positive_int in
    (* TODO: test for this too. *)
    let ord_partial_fill_okay = true in
    let order_args =
      {
        ord_desired_asset;
        ord_desired_amt;
        ord_from = balance;
        ord_max_pol_fee;
        ord_partial_fill_okay;
      }
    in
    return (Order_inline (`Order_inline order_args))
  and gen_order_wanting_asset wanted_asset balance =
    let bal_asset = Apply.bal_asset balance in
    let* balance =
      if bal_asset = wanted_asset then
        gen_balance_with_asset_different_from wanted_asset
      else return balance
    in
    let* ord_desired_amt = gen_positive_int in
    let* ord_max_pol_fee = gen_positive_int in
    (* TODO: test for this too. *)
    let ord_partial_fill_okay = true in
    let order_args =
      {
        ord_desired_asset = wanted_asset;
        ord_desired_amt;
        ord_from = balance;
        ord_max_pol_fee;
        ord_partial_fill_okay;
      }
    in
    return (Order_inline (`Order_inline order_args))
  and gen_commit left_order right_order =
    let* commit_id = gen_positive_int in
    return (Commit_inline (`Commit_inline (commit_id, left_order, right_order)))
  and gen_transaction depth =
    if depth <= 0 then
      let* balance = gen_balance () in
      return (Withdraw balance)
    else
      let* balance = gen_balance () in
      frequency
        [
          (1, return (Withdraw balance));
          (10, gen_order_flow balance (depth - 1));
          (10, gen_join_flow balance (depth - 1));
        ]
  and gen_order_flow balance depth =
    let* order = gen_order_from_balance balance in
    if depth <= 0 then
      let cancelled_balance = Cancel order in
      return (Withdraw cancelled_balance)
    else
      frequency
        [
          ( 1,
            let cancelled_balance = Cancel order in
            return (Withdraw cancelled_balance) );
          (10, gen_commit_flow order (depth - 1));
        ]
  and gen_commit_flow order1 depth =
    let order1_offered_asset = Apply.ord_asset order1 in
    let order1_desired_asset = Apply.ord_desired_asset order1 in
    let* balance2 = gen_balance_with_asset order1_desired_asset in
    let* balance2 =
      let b2a = Apply.bal_asset balance2 in
      if b2a = order1_offered_asset then
        gen_balance_with_asset_different_from order1_offered_asset
      else return balance2
    in
    let* order2 = gen_order_wanting_asset order1_offered_asset balance2 in
    let* commit = gen_commit order1 order2 in
    if depth <= 0 then
      let* side = oneofl [ `Left; `Right ] in
      let balance =
        match side with
        | `Left -> Commit_left_filled_to_bal commit
        | `Right -> Commit_right_filled_to_bal commit
      in
      return (Withdraw balance)
    else
      frequency
        [
          ( 1,
            let* side = oneofl [ `Left; `Right ] in
            let balance =
              match side with
              | `Left -> Commit_left_filled_to_bal commit
              | `Right -> Commit_right_filled_to_bal commit
            in
            return (Withdraw balance) );
          ( 1,
            let* side = oneofl [ `Left; `Right ] in
            let order =
              match side with
              | `Left -> Commit_left_excess_to_order commit
              | `Right -> Commit_right_excess_to_order commit
            in
            gen_order_flow_from_existing order (depth - 1) );
        ]
  and gen_order_flow_from_existing order depth =
    if depth <= 0 then
      let cancelled_balance = Cancel order in
      return (Withdraw cancelled_balance)
    else
      frequency
        [
          ( 2,
            let cancelled_balance = Cancel order in
            return (Withdraw cancelled_balance) );
          (10, gen_commit_flow order (depth - 1));
        ]
  and gen_join_flow balance1 depth =
    let* balance2 = gen_balance () in
    let joined_balance = Join (balance1, balance2) in
    if depth <= 0 then return (Withdraw joined_balance)
    else gen_transaction (depth - 1)
  in
  gen_transaction 1000

let _test_complex =
  QCheck_ounit.to_ounit2_test
    (QCheck2.Test.make ~count:1000 ~name:"Property test valid creation"
       ~print:(fun (c, s) ->
         Format.asprintf "App: %a@.State: %a@." Applicative.pp_withdraw c
           State.pp s)
       (let* x = gen in
        return (x, State.state_of_app (`Withdraw x)))
       (fun (x, state) ->
         let State.{ interim; orders; withdrawable; _ } =
           Apply.apply_withdraw state x
         in
         State.Order_state.iter
           (fun _ v -> if 0 > v then failwith "Negative interim")
           interim;
         State.Order_state.iter
           (fun _ v -> if 0 > v then failwith "Negative order value")
           orders;
         State.Asset_owner.iter
           (fun _ v -> if 0 > v then failwith "Negative withdrawable")
           withdrawable;
         true))

let _test_simple _ =
  let w =
    Withdraw
      (Commit_left_filled_to_bal
         (Commit_inline
            (`Commit_inline
               ( 1,
                 Order_inline
                   (`Order_inline
                      {
                        ord_desired_asset = "AAAA";
                        ord_desired_amt = 1;
                        ord_from =
                          Balance_inline
                            (`Bal_inline
                               {
                                 bal_ms_ts = 0;
                                 bal_owner = "AAA";
                                 bal_asset = "AAA";
                                 bal_amt = 1;
                               });
                               ord_max_pol_fee = 0;
                               ord_partial_fill_okay = true;
                      }),
                 Order_inline
                   (`Order_inline
                      {
                        ord_desired_asset = "AAA";
                        ord_desired_amt = 1;
                        ord_from =
                          Balance_inline
                            (`Bal_inline
                               {
                                 bal_ms_ts = 0;
                                 bal_owner = "AAA";
                                 bal_asset = "AAAA";
                                 bal_amt = 1;
                               });
                               ord_max_pol_fee = 0;
                               ord_partial_fill_okay = true;
                      }) ))))
  in
  let s = State.state_of_app (`Withdraw w) in
  Format.eprintf "Storage after: %a@." State.pp (Apply.apply_withdraw s w)

let () =
  let open OUnit2 in
  run_test_tt_main ("Passport tests" >: _test_complex)
