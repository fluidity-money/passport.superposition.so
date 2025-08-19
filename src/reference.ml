#!/usr/bin/env -S ocaml -w +A -w -4-42

(* NOTE: We overload the addition/subtraction operators in this code to
 * make them checked! *)

#use "topfind"

#require "ounit2"

#require "qcheck-core"

#require "qcheck-core.runner"

#require "qcheck-ounit"

exception Checked_add of int * int * int

exception Checked_sub of int * int * int

let (-) x y =
  let z = x - y in
  if 0 > z then raise (Checked_sub (x, y, z)) else z

let (+) x y =
  let z = x + y in
  if 0 > z then raise (Checked_add (x, y, z)) else z

module Applicative = struct
  type bal
  type commit
  type order
  type withdrawn

  type bal_args = {
    bal_ms_ts : int;
    bal_owner : string;
    bal_asset : string;
    bal_amt : int;
  }

  and order_args = {
    ord_desired_asset : string;
    ord_desired_amt : int;
    ord_from : bal t;
  }

  and 'a t =
    | Balance : bal_args -> bal t
    | Order : order_args -> order t
    | Commit : (int * order t * order t) -> commit t
    | Commit_left_filled_to_bal : commit t -> bal t
    | Commit_right_filled_to_bal : commit t -> bal t
    | Commit_left_excess_to_order : commit t -> order t
    | Commit_right_excess_to_order : commit t -> order t
    | Withdraw : bal t -> withdrawn t
    | Cancel : order t -> bal t
    | Join : bal t * bal t -> bal t
    | Stored_onchain : hash -> 'a t

  let rec pp_bal fmt x =
    match x with
    | Balance { bal_ms_ts; bal_owner; bal_asset; bal_amt } ->
        Format.fprintf fmt
          "Balance { bal_ms_ts = %d;  bal_owner = %s; bal_asset = %s ; bal_amt \
           = %d }"
          bal_ms_ts bal_owner bal_asset bal_amt
    | Commit_left_filled_to_bal c ->
        Format.fprintf fmt "Commit_left_filled_to_bal { commit = %a }" pp_commit
          c
    | Commit_right_filled_to_bal c ->
        Format.fprintf fmt "Commit_right_filled_to_bal { commit = %a }"
          pp_commit c
    | Cancel o -> Format.fprintf fmt "Cancel %a" pp_order o
    | Join (x, y) -> Format.fprintf fmt "Join (%a, %a)" pp_bal x pp_bal y

  and pp_order fmt = function
    | Order { ord_desired_asset; ord_desired_amt; ord_from } ->
        Format.fprintf fmt
          "Order { ord_desired_asset = %s ; ord_desired_amt = %d ; ord_from = \
           (%a)"
          ord_desired_asset ord_desired_amt pp_bal ord_from
    | Commit_left_excess_to_order c ->
        Format.fprintf fmt "Commit_left_excess_to_order (%a)" pp_commit c
    | Commit_right_excess_to_order c ->
        Format.fprintf fmt "Commit_right_excess_to_order (%a)" pp_commit c

  and pp_commit fmt (Commit (ts, l, r)) =
    Format.fprintf fmt "Commit (%d, %a, %a)" ts pp_order l pp_order r

  and pp_withdraw fmt (Withdraw b) = Format.fprintf fmt "Withdraw (%a)" pp_bal b

  (** This module is used for indiscriminate fuzzing of access control. *)
  module Easy = struct
    type easy_balance =
      [ `Balance of int * string * string * int
      | `Commit_left_filled_to_balance of easy_commit
      | `Commit_right_filled_to_balance of easy_commit ]

    and easy_order =
      [ `Order of string * int * easy_balance
      | `Commit_left_excess_to_order of easy_commit
      | `Commit_right_excess_to_order of easy_commit ]

    and easy_commit = [ `Commit of int * easy_order * easy_order ]
    and easy_withdraw = [ `Withdraw of easy_balance ] [@@deriving qcheck]
  end

  let rec of_easy_bal = function
    | `Balance (bal_ms_ts, bal_owner, bal_asset, bal_amt) ->
        Balance { bal_ms_ts; bal_owner; bal_asset; bal_amt }
    | `Commit_left_filled_to_balance c ->
        Commit_left_filled_to_bal (of_easy_commit c)
    | `Commit_right_filled_to_balance c ->
        Commit_right_filled_to_bal (of_easy_commit c)

  and of_easy_commit (`Commit (ts, l, r)) =
    Commit (ts, of_easy_order l, of_easy_order r)

  and of_easy_order = function
    | `Order (ord_desired_asset, ord_desired_amt, from) ->
        Order
          { ord_desired_asset; ord_desired_amt; ord_from = of_easy_bal from }
    | `Commit_left_excess_to_order c ->
        Commit_left_excess_to_order (of_easy_commit c)
    | `Commit_right_excess_to_order c ->
        Commit_right_excess_to_order (of_easy_commit c)
end

module State = struct
  module Orders_key = struct
    type t = {
      desired_asset : string;
      owner : string;
      desired_amt : int;
      from_asset : string;
    }

    let pp fmt { desired_asset; owner; desired_amt; from_asset } =
      Format.fprintf fmt
        "{ desired_asset = %s ; owner = %s ; desired_amt = %d ; from_asset = \
         %s }"
        desired_asset owner desired_amt from_asset
  end

  module Order_state = Map.Make (struct
    type t = Orders_key.t

    let compare
        Orders_key.
          { desired_asset = x; owner = y; desired_amt = z; from_asset = a }
        Orders_key.
          { desired_asset = x'; owner = y'; desired_amt = z'; from_asset = a' }
        =
      match
        ( String.compare x x',
          String.compare y y',
          Int.compare z z',
          String.compare a a' )
      with
      | 0, 0, 0, x | 0, 0, x, _ | 0, x, _, _ | x, _, _, _ -> x
  end)

  module Asset_owner_key = struct
    type t = { asset : string; owner : string }

    let pp fmt { asset; owner } =
      Format.fprintf fmt "{ asset = %s ; owner = %s }" asset owner
  end

  module Asset_owner = Map.Make (struct
    type t = Asset_owner_key.t

    let compare Asset_owner_key.{ asset = x; owner = y }
        Asset_owner_key.{ asset = x'; owner = y' } =
      match (String.compare x x', String.compare y y') with 0, x | x, _ -> x
  end)

  type t = {
    orders : int Order_state.t;
    withdrawable : int Asset_owner.t;
    interim : int Asset_owner.t;
  }

  let pp_orders fmt =
    Order_state.iter (fun k v ->
        Format.fprintf fmt "{ (%a) : %d }, " Orders_key.pp k v)

  let pp_asset_owner fmt =
    Asset_owner.iter (fun Asset_owner_key.{ asset; owner } v ->
        Format.fprintf fmt "{ (%s, %s): %d }, " asset owner v)

  let pp fmt { orders; withdrawable; interim } =
    Format.fprintf fmt "{ orders : [%a], withdrawable: [%a], interim: [%a] }"
      pp_orders orders pp_asset_owner withdrawable pp_asset_owner interim

  let empty =
    {
      orders = Order_state.empty;
      withdrawable = Asset_owner.empty;
      interim = Asset_owner.empty;
    }

  let with_bals bals =
    {
      empty with
      withdrawable =
        List.fold_left
          (fun acc ((owner, asset), v) ->
            Asset_owner.add Asset_owner_key.{ owner; asset } v acc)
          empty.withdrawable bals;
    }

  let state_of_app =
    let open Applicative in
    let rec loop_bal ({ withdrawable; _ } as acc) = function
      | Balance { bal_owner; bal_asset; bal_amt; _ } ->
          {
            acc with
            withdrawable =
              Asset_owner.update
                Asset_owner_key.{ asset = bal_asset; owner = bal_owner }
                (function Some v -> Some (v + bal_amt) | None -> Some bal_amt)
                withdrawable;
          }
      | Commit_left_filled_to_bal f | Commit_right_filled_to_bal f ->
          loop_commit acc f
      | Join (l, r) -> loop_bal (loop_bal acc l) r
      | Cancel o -> loop_order acc o
      | _ -> assert false
    and loop_commit acc = function
      | Commit (_, l, r) -> loop_order (loop_order acc l) r
      | _ -> assert false
    and loop_order acc = function
      | Order { ord_from; _ } -> loop_bal acc ord_from
      | Cancel f -> loop_order acc f
      | Commit_left_excess_to_order f | Commit_right_excess_to_order f ->
          loop_commit acc f
      | _ -> assert false
    and loop_withdraw acc = function
      | Withdraw f -> loop_bal acc f
      | _ -> assert false
    in
    function
    | `Balance b -> loop_bal empty b
    | `Withdraw w -> loop_withdraw empty w
    | `Order o -> loop_order empty o
    | `Commit c -> loop_commit empty c
end

module Apply = struct
  (* We can afford to ignore warnings here since the type isn't allowing
   * situations where this unpacking happens, unless we add extra types. *)
  [@@@warning "-8"]

  let rec bal_owner = function
    | Applicative.(Balance { bal_owner; _ }) -> bal_owner
    | Applicative.(Commit_left_filled_to_bal (Commit (_, x, _)))
    | Applicative.(Commit_right_filled_to_bal (Commit (_, _, x))) ->
        ord_owner x
    | Applicative.Cancel o -> ord_owner o

  and bal_amt = function
    | Applicative.(Balance { bal_amt; _ }) -> bal_amt
    | Applicative.(Commit_left_filled_to_bal c) -> commit_left_amt_filled c
    | Applicative.(Commit_right_filled_to_bal c) -> commit_right_amt_filled c
    | Applicative.Cancel o -> ord_amt o

  and bal_asset = function
    | Applicative.(Balance { bal_asset; _ }) -> bal_asset
    | Applicative.(Commit_left_filled_to_bal (Commit (_, x, _)))
    | Applicative.(Commit_right_filled_to_bal (Commit (_, _, x))) ->
        ord_desired_asset x
    | Applicative.Cancel o -> ord_asset o

  and ord_desired_amt = function
    | Applicative.(Order { ord_desired_amt; _ }) -> ord_desired_amt
    | Applicative.Commit_left_excess_to_order c -> commit_left_amt_unfilled c
    | Applicative.Commit_right_excess_to_order c -> commit_right_amt_unfilled c

  and ord_desired_asset = function
    | Applicative.(Order { ord_desired_asset; _ }) -> ord_desired_asset
    | Applicative.(Commit_left_excess_to_order (Commit (_, x, _)))
    | Applicative.(Commit_right_excess_to_order (Commit (_, _, x))) ->
        ord_desired_asset x

  and ord_amt = function
    | Applicative.(Order { ord_from; _ }) -> bal_amt ord_from
    | Applicative.(Commit_left_excess_to_order c) -> commit_left_amt_unfilled c
    | Applicative.(Commit_right_excess_to_order c) ->
        commit_right_amt_unfilled c

  and ord_owner = function
    | Applicative.(Order { ord_from; _ }) -> bal_owner ord_from
    | Applicative.(Commit_left_excess_to_order (Commit (_, x, _)))
    | Applicative.(Commit_right_excess_to_order (Commit (_, _, x))) ->
        ord_owner x

  and ord_asset = function
    | Applicative.(Order { ord_from; _ }) -> bal_asset ord_from
    | Applicative.(Commit_left_excess_to_order (Commit (_, x, _)))
    | Applicative.(Commit_right_excess_to_order (Commit (_, _, x))) ->
        ord_asset x

  and withdraw_asset Applicative.(Withdraw w) = bal_asset w
  and withdraw_amt Applicative.(Withdraw w) = bal_amt w

  and commit_left_amt_filled (Applicative.Commit (_, l, r)) =
    min (ord_desired_amt r) (ord_amt l)

  and commit_right_amt_filled (Applicative.Commit (_, l, r)) =
    min (ord_desired_amt l) (ord_amt r)

  and commit_left_amt_unfilled (Applicative.Commit (_, l, r)) =
    try ord_amt l - ord_desired_amt r with Checked_sub (_, _, _) -> 0

  and commit_right_amt_unfilled (Applicative.Commit (_, l, r)) =
    try ord_amt r - ord_desired_amt l with Checked_sub (_, _, _) -> 0

  and apply_bal (State.{ withdrawable; interim; _ } as s) = function
    | Applicative.(Balance { bal_owner; bal_asset; bal_amt; _ }) ->
        if bal_amt <= 0 then invalid_arg "Balance is zero";
        let k =
          State.Asset_owner_key.{ asset = bal_asset; owner = bal_owner }
        in
        State.
          {
            s with
            interim =
              Asset_owner.update k
                (function Some v -> Some (v + bal_amt) | None -> Some bal_amt)
                interim;
            withdrawable =
              Asset_owner.update k
                (function
                  | Some withdrawable when bal_amt > withdrawable ->
                      invalid_arg "Not enough withdrawable"
                  | Some withdrawable -> Some (withdrawable - bal_amt)
                  | None -> invalid_arg "No withdrawable amount")
                withdrawable;
          }
    | Applicative.(Commit_left_filled_to_bal c)
    | Applicative.(Commit_right_filled_to_bal c) ->
        apply_commit s c
    | Applicative.Cancel o ->
        let from_asset = ord_asset o in
        let desired_asset = ord_desired_asset o in
        let desired_amt = ord_desired_amt o in
        let amt = ord_amt o in
        let owner = ord_owner o in
        let s = apply_order s o in
        let State.{ orders; interim; _ } = s in
        let k =
          State.Orders_key.{ desired_asset; owner; desired_amt; from_asset }
        in
        (* In situations where the user would try to cancel a position
         * with 0 in it, we don't need to adjust the state: *)
        if ord_desired_amt o = 0 then s
        else
          State.
            {
              s with
              orders =
                Order_state.update k
                  (function
                    | Some v when amt > v ->
                        invalid_arg
                          (Format.asprintf
                             "Order amount less than cancel, updating using \
                              key (%a), tried to take amount %d, only have %d"
                             State.Orders_key.pp k amt v)
                    | Some v -> Some (v - amt)
                    | None as n when amt = 0 -> n
                    | None ->
                        invalid_arg
                          (Format.asprintf
                             "Order amount empty, state: (%a). Tried key (%a). \
                              Hoping to spend %d"
                             State.pp s Orders_key.pp k amt))
                  orders;
              interim =
                Asset_owner.update
                  Asset_owner_key.{ asset = from_asset; owner }
                  (function Some v -> Some (v + amt) | None -> Some amt)
                  interim;
            }

  and apply_order s order =
    let from_asset = ord_asset order in
    let owner = ord_owner order in
    let amt = ord_amt order in
    let desired_asset = ord_desired_asset order in
    let desired_amt = ord_desired_amt order in
    if from_asset = desired_asset then
      invalid_arg "Same asset desired as supplied";
    (* We don't actually do anything unless this is a fresh Order, since
     * the Commit step will apply the order allocation for
     * Commit_left_excess_to_order and Commit_right_excess_to_order. *)
    match order with
    | Applicative.(Order { ord_from; _ }) ->
        let s = apply_bal s ord_from in
        let State.{ orders; interim; _ } = s in
        State.
          {
            s with
            orders =
              State.Order_state.update
                Orders_key.{ desired_asset; owner; desired_amt; from_asset }
                (function Some v -> Some (v + amt) | None -> Some amt)
                orders;
            interim =
              Asset_owner.update
                Asset_owner_key.{ asset = from_asset; owner }
                (function
                  | Some v when amt > v ->
                      invalid_arg
                        (Format.sprintf
                           "Not enough interim for order, have %d, want %d" v
                           amt)
                  | Some v -> Some (v - amt)
                  | None -> invalid_arg "no balance for interim order")
                interim;
          }
    | Applicative.Commit_left_excess_to_order c
    | Applicative.Commit_right_excess_to_order c ->
        apply_commit s c

  and apply_commit s (Applicative.Commit (_, l, r) as c) =
    let l_from_asset = ord_asset l in
    let r_from_asset = ord_asset r in
    let l_desired_amt = ord_desired_amt l in
    let r_desired_amt = ord_desired_amt r in
    let l_desired_asset = ord_desired_asset l in
    let r_desired_asset = ord_desired_asset r in
    if not (String.equal (ord_asset r) l_desired_asset) then
      invalid_arg "Incorrect desired left asset";
    if not (String.equal (ord_asset l) r_desired_asset) then
      invalid_arg "Incorrect desired right asset";
    let l_filled = commit_left_amt_filled c in
    let r_filled = commit_right_amt_filled c in
    let l_unfilled = commit_left_amt_unfilled c in
    let r_unfilled = commit_right_amt_unfilled c in
    let l_owner = ord_owner l in
    let r_owner = ord_owner r in
    let s = apply_order (apply_order s l) r in
    let State.{ orders; interim; _ } = s in
    let set_order x y z = State.Order_state.update x (fun _ -> Some y) z in
    let set_interim x y z = State.Asset_owner.update x (fun _ -> Some y) z in
    (* When a user has their order partially filled, we must also destroy
     * their previous order and make a new one with the remainder that
     * they want to fill, so that it can be addressable later. *)
    let l_desired_unfilled = commit_left_amt_unfilled c in
    let r_desired_unfilled = commit_right_amt_unfilled c in
    let l_filled_key =
      State.Orders_key.
        {
          desired_asset = r_desired_asset;
          desired_amt = r_desired_amt;
          owner = r_owner;
          from_asset = r_from_asset;
        }
    in
    let r_filled_key =
      State.Orders_key.
        {
          desired_asset = l_desired_asset;
          desired_amt = l_desired_amt;
          owner = l_owner;
          from_asset = l_from_asset;
        }
    in
    let orders = set_order l_filled_key 0 (set_order r_filled_key 0 orders) in
    (* We only set the orders for the new unfilled amounts if there's
     * something to be set: *)
    let orders =
      if l_desired_unfilled > 0 then
        set_order
          State.Orders_key.
            {
              desired_asset = l_desired_asset;
              desired_amt = l_desired_unfilled;
              owner = l_owner;
              from_asset = l_from_asset;
            }
          l_unfilled orders
      else orders
    in
    let orders =
      if r_desired_unfilled > 0 then
        set_order
          State.Orders_key.
            {
              desired_asset = r_desired_asset;
              desired_amt = r_desired_unfilled;
              owner = r_owner;
              from_asset = r_from_asset;
            }
          r_unfilled orders
      else orders
    in
    State.
      {
        s with
        interim =
          set_interim
            Asset_owner_key.{ asset = r_desired_asset; owner = r_owner }
            r_filled
            (set_interim
               Asset_owner_key.{ asset = l_desired_asset; owner = l_owner }
               l_filled interim);
        orders;
      }

  and apply_withdraw s (Applicative.Withdraw b) =
    let owner = bal_owner b in
    let asset = bal_asset b in
    let amt = bal_amt b in
    let s = apply_bal s b in
    let State.{ interim; withdrawable; _ } = s in
    let k = State.Asset_owner_key.{ asset; owner } in
    State.
      {
        s with
        interim =
          Asset_owner.update k
            (function
              | Some v when amt > v ->
                  invalid_arg
                    (Format.asprintf
                       "Not enough interim state (%a), key (%a), needed: %d, \
                        only have %d"
                       State.pp s State.Asset_owner_key.pp k amt v)
              | Some v -> Some (v - amt)
              | None -> invalid_arg "No interim balance")
            interim;
        withdrawable =
          Asset_owner.update k
            (function Some v -> Some (v + amt) | None -> Some amt)
            withdrawable;
      }
end

module Apply_tests = struct
  open Applicative

  let gen =
    (* Claude generated: *)
    let open QCheck2.Gen in
    let assets = [ "ERIK"; "IVAN"; "OGOUS"; "ELI" ] in
    let gen_positive_int = int_bound (Int32.to_int Int32.max_int) in
    let gen_timestamp = map (fun x -> abs x) int in
    let gen_owner = oneofl [ "Erik"; "Ivan"; "Ogous"; "Eli" ] in

    let rec gen_balance_args () =
      let* bal_ms_ts = gen_timestamp in
      let* bal_asset = oneofl assets in
      let* bal_owner = gen_owner in
      let* bal_amt = gen_positive_int in
      return { bal_ms_ts; bal_asset; bal_owner; bal_amt }
    and gen_balance () =
      let* args = gen_balance_args () in
      return (Balance args)
    and gen_balance_with_asset asset =
      let* bal_ms_ts = gen_timestamp in
      let* bal_owner = gen_owner in
      let* bal_amt = gen_positive_int in
      return (Balance { bal_ms_ts; bal_asset = asset; bal_owner; bal_amt })
    and gen_order_from_balance balance =
      let bal_asset = Apply.bal_asset balance in
      let* ord_desired_asset = oneofl (List.filter (( != ) bal_asset) assets) in
      let* ord_desired_amt = gen_positive_int in
      let order_args =
        { ord_desired_asset; ord_desired_amt; ord_from = balance }
      in
      return (Order order_args)
    and gen_order_wanting_asset wanted_asset balance =
      let bal_asset = Apply.bal_asset balance in
      if bal_asset = wanted_asset then
        failwith "Cannot create order wanting same asset as balance"
      else
        let* ord_desired_amt = gen_positive_int in
        let order_args =
          {
            ord_desired_asset = wanted_asset;
            ord_desired_amt;
            ord_from = balance;
          }
        in
        return (Order order_args)
    and gen_commit left_order right_order =
      let* commit_id = gen_positive_int in
      return (Commit (commit_id, left_order, right_order))
    and gen_transaction depth =
      if depth <= 0 then
        let* balance = gen_balance () in
        return (Withdraw balance)
      else
        let* balance = gen_balance () in
        frequency
          [
            (3, return (Withdraw balance));
            (2, gen_order_flow balance (depth - 1));
            (1, gen_join_flow balance (depth - 1));
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
            (5, gen_commit_flow order (depth - 1));
          ]
    and gen_commit_flow order1 depth =
      let order1_desired_asset = Apply.ord_desired_asset order1 in
      let order1_offered_asset = Apply.ord_asset order1 in
      let* balance2 = gen_balance_with_asset order1_desired_asset in
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
            ( 2,
              let* side = oneofl [ `Left; `Right ] in
              let balance =
                match side with
                | `Left -> Commit_left_filled_to_bal commit
                | `Right -> Commit_right_filled_to_bal commit
              in
              return (Withdraw balance) );
            ( 2,
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
            (1, gen_commit_flow order (depth - 1));
          ]
    and gen_join_flow balance1 depth =
      let* balance2 = gen_balance () in
      let joined_balance = Join (balance1, balance2) in
      if depth <= 0 then return (Withdraw joined_balance)
      else gen_transaction (depth - 1)
    in
    gen_transaction 100_000
end

let () =
  let open OUnit2 in
  let open QCheck2.Gen in
  run_test_tt_main
    ("Passport tests"
    >: QCheck_ounit.to_ounit2_test
         (QCheck2.Test.make ~count:10_000_000 ~name:"Property test valid creation"
            ~print:(fun (c, s) ->
              Format.asprintf "App: %a@.State: %a@." Applicative.pp_withdraw c
                State.pp s)
            (let* x = Apply_tests.gen in
             return (x, State.state_of_app (`Withdraw x)))
            (fun (x, state) ->
              let State.{ interim; orders; withdrawable; _ } =
                Apply.apply_withdraw state x
              in
              State.Asset_owner.iter
                (fun _ v -> if 0 > v then failwith "Negative interim")
                interim;
              State.Order_state.iter
                (fun _ v -> if 0 > v then failwith "Negative order value")
                orders;
              State.Asset_owner.iter
                (fun _ v -> if 0 > v then failwith "Negative withdrawable")
                withdrawable;
              true)))
