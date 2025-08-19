(* We can afford to ignore warnings here since the type isn't allowing
 * situations where this unpacking happens, unless we add extra types. *)
[@@@warning "-8"]

exception Checked_add of int * int * int
exception Checked_sub of int * int * int

let ( - ) x y =
  let z = x - y in
  if 0 > z then raise (Checked_sub (x, y, z)) else z

let ( + ) x y =
  let z = x + y in
  if 0 > z then raise (Checked_add (x, y, z)) else z

let rec commit_left_filled_to_bal_owner =
  let open Applicative in
  function
  | Commit_inline (`Commit_inline (_, x, _)) -> ord_owner x
  | Commit_onchain (`Commit_onchain _) -> failwith "TODO"

and commit_right_filled_to_bal_owner =
  let open Applicative in
  function
  | Commit_inline (`Commit_inline (_, _, x)) -> ord_owner x
  | Commit_onchain (`Commit_onchain _) -> failwith "TODO"

and bal_inline_owner Applicative.(`Bal_inline { bal_owner; _ }) = bal_owner
and bal_onchain_owner (`Bal_onchain _) = failwith "TODO"

and bal_owner =
  let open Applicative in
  function
  | Balance_inline b -> bal_inline_owner b
  | Balance_onchain b -> bal_onchain_owner b
  | Commit_left_filled_to_bal c -> commit_left_filled_to_bal_owner c
  | Commit_right_filled_to_bal c -> commit_right_filled_to_bal_owner c
  | Cancel o -> ord_owner o

and bal_inline_amt Applicative.(`Bal_inline { bal_amt; _ }) = bal_amt
and bal_onchain_amt _ = failwith "TODO"

and commit_left_amt_filled =
  let open Applicative in
  function
  | Commit_inline (`Commit_inline _ as c) -> commit_left_inline_amt_filled c
  | Commit_onchain _ -> failwith "TODO"

and commit_right_amt_filled =
  let open Applicative in
  function
  | Commit_inline (`Commit_inline _ as c) -> commit_right_inline_amt_filled c
  | Commit_onchain _ -> failwith "TODO"

and bal_amt =
  let open Applicative in
  function
  | Balance_inline b -> bal_inline_amt b
  | Balance_onchain b -> bal_onchain_amt b
  | Commit_left_filled_to_bal c -> commit_left_amt_filled c
  | Commit_right_filled_to_bal c -> commit_right_amt_filled c
  | Cancel o -> ord_amt o

and bal_inline_asset Applicative.(`Bal_inline { bal_asset; _ }) = bal_asset
and bal_onchain_asset _ = failwith "TODO"

and commit_inline_left_desired_asset (`Commit_inline (_, x, _)) =
  ord_desired_asset x

and commit_inline_right_desired_asset (`Commit_inline (_, _, x)) =
  ord_desired_asset x

and commit_onchain_left_desired_asset (`Commit_onchain _) = failwith "TODO"
and commit_onchain_right_desired_asset (`Commit_onchain _) = failwith "TODO"

and commit_left_desired_asset = function
  | Applicative.Commit_inline o -> commit_inline_left_desired_asset o
  | Applicative.Commit_onchain o -> commit_onchain_left_desired_asset o

and commit_right_desired_asset = function
  | Applicative.Commit_inline o -> commit_inline_right_desired_asset o
  | Applicative.Commit_onchain o -> commit_onchain_right_desired_asset o

and bal_asset =
  let open Applicative in
  function
  | Balance_inline b -> bal_inline_asset b
  | Balance_onchain b -> bal_onchain_asset b
  | Commit_left_filled_to_bal c -> commit_left_desired_asset c
  | Commit_right_filled_to_bal c -> commit_right_desired_asset c
  | Cancel o -> ord_asset o

and ord_inline_desired_amt Applicative.(`Order_inline { ord_desired_amt; _ }) =
  ord_desired_amt

and ord_onchain_desired_amt (`Order_onchain _) = failwith "TODO"

and commit_left_amt_unfilled =
  let open Applicative in
  function
  | Commit_inline o -> commit_left_inline_amt_unfilled o
  | Commit_onchain _ -> failwith "TODO"

and commit_right_amt_unfilled =
  let open Applicative in
  function
  | Commit_inline o -> commit_right_inline_amt_unfilled o
  | Commit_onchain _ -> failwith "TODO"

and ord_desired_amt =
  let open Applicative in
  function
  | Order_inline o -> ord_inline_desired_amt o
  | Order_onchain o -> ord_onchain_desired_amt o
  | Commit_left_excess_to_order c -> commit_left_amt_unfilled c
  | Commit_right_excess_to_order c -> commit_right_amt_unfilled c

and ord_inline_desired_asset
    Applicative.(`Order_inline { ord_desired_asset; _ }) =
  ord_desired_asset

and ord_onchain_desired_asset (`Order_onchain _) = failwith "TODO"

and ord_desired_asset =
  let open Applicative in
  function
  | Order_inline o -> ord_inline_desired_asset o
  | Order_onchain o -> ord_onchain_desired_asset o
  | Commit_left_excess_to_order c -> commit_left_desired_asset c
  | Commit_right_excess_to_order c -> commit_right_desired_asset c

and order_inline_amt Applicative.(`Order_inline { ord_from; _ }) =
  bal_amt ord_from

and order_onchain_amt (`Order_onchain _) = failwith "TODO"

and ord_amt =
  let open Applicative in
  function
  | Order_inline o -> order_inline_amt o
  | Order_onchain o -> order_onchain_amt o
  | Commit_left_excess_to_order c -> commit_left_amt_unfilled c
  | Commit_right_excess_to_order c -> commit_right_amt_unfilled c

and order_inline_owner Applicative.(`Order_inline { ord_from; _ }) =
  bal_owner ord_from

and order_onchain_owner (`Order_onchain _) = failwith "TODO"
and commit_inline_left_owner (`Commit_inline (_, x, _)) = ord_owner x
and commit_inline_right_owner (`Commit_inline (_, _, x)) = ord_owner x
and commit_onchain_left_owner (`Commit_onchain _) = failwith "TODO"
and commit_onchain_right_owner (`Commit_onchain _) = failwith "TODO"

and commit_left_owner = function
  | Applicative.Commit_inline o -> commit_inline_left_owner o
  | Applicative.Commit_onchain o -> commit_onchain_left_owner o

and commit_right_owner = function
  | Applicative.Commit_inline o -> commit_inline_right_owner o
  | Applicative.Commit_onchain o -> commit_onchain_right_owner o

and ord_owner =
  let open Applicative in
  function
  | Order_inline o -> order_inline_owner o
  | Order_onchain o -> order_onchain_owner o
  | Commit_left_excess_to_order o -> commit_left_owner o
  | Commit_right_excess_to_order o -> commit_right_owner o

and commit_left_asset =
  let open Applicative in
  function
  | Commit_inline (`Commit_inline (_, _, x)) -> ord_asset x
  | Commit_onchain _ -> failwith "TODO"

and commit_right_asset =
  let open Applicative in
  function
  | Commit_inline (`Commit_inline (_, _, x)) -> ord_asset x
  | Commit_onchain _ -> failwith "TODO"

and order_inline_asset Applicative.(`Order_inline { ord_from; _ }) =
  bal_asset ord_from

and order_onchain_asset (`Order_onchain _) = failwith "TODO"

and ord_asset =
  let open Applicative in
  function
  | Order_inline o -> order_inline_asset o
  | Order_onchain o -> order_onchain_asset o
  | Commit_left_excess_to_order o -> commit_left_asset o
  | Commit_right_excess_to_order o -> commit_right_asset o

and withdraw_asset Applicative.(Withdraw w) = bal_asset w
and withdraw_amt Applicative.(Withdraw w) = bal_amt w

and commit_left_inline_amt_filled (`Commit_inline (_, l, r)) =
  min (ord_desired_amt r) (ord_amt l)

and commit_right_inline_amt_filled (`Commit_inline (_, l, r)) =
  min (ord_desired_amt l) (ord_amt r)

and commit_left_inline_amt_unfilled (`Commit_inline (_, l, r)) =
  try ord_amt l - ord_desired_amt r with Checked_sub (_, _, _) -> 0

and commit_right_inline_amt_unfilled (`Commit_inline (_, l, r)) =
  try ord_amt r - ord_desired_amt l with Checked_sub (_, _, _) -> 0

and apply_bal_inline (State.{ withdrawable; interim; _ } as s)
    Applicative.(`Bal_inline { bal_owner; bal_asset; bal_amt; _ }) =
  if bal_amt <= 0 then invalid_arg "Balance is zero";
  let k = State.Asset_owner_key.{ asset = bal_asset; owner = bal_owner } in
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

and apply_cancel s (Applicative.Cancel o) =
  let from_asset = ord_asset o in
  let desired_asset = ord_desired_asset o in
  let desired_amt = ord_desired_amt o in
  let amt = ord_amt o in
  let owner = ord_owner o in
  let s = apply_order s o in
  let State.{ orders; interim; _ } = s in
  let k = State.Orders_key.{ desired_asset; owner; desired_amt; from_asset } in
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
                       "Order amount less than cancel, updating using key \
                        (%a), tried to take amount %d, only have %d"
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

and apply_bal s = function
  | Applicative.(Balance_inline b) -> apply_bal_inline s b
  | Applicative.(Balance_onchain _) -> failwith "TODO"
  | Applicative.(Commit_left_filled_to_bal c)
  | Applicative.(Commit_right_filled_to_bal c) ->
      apply_commit s c
  | Applicative.Cancel _ as c -> apply_cancel s c

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
  let open Applicative in
  match order with
  | Order_inline (`Order_inline { ord_from; _ }) ->
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
                         "Not enough interim for order, have %d, want %d" v amt)
                | Some v -> Some (v - amt)
                | None -> invalid_arg "no balance for interim order")
              interim;
        }
  | Order_onchain (`Order_onchain _) -> failwith "TODO"
  | Commit_left_excess_to_order c | Commit_right_excess_to_order c ->
      apply_commit s c

and apply_inline_commit s (`Commit_inline (_, l, r) as c) =
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
  let l_filled = commit_left_inline_amt_filled c in
  let r_filled = commit_right_inline_amt_filled c in
  let l_unfilled = commit_left_inline_amt_unfilled c in
  let r_unfilled = commit_right_inline_amt_unfilled c in
  let l_owner = ord_owner l in
  let r_owner = ord_owner r in
  let s = apply_order (apply_order s l) r in
  let State.{ orders; interim; _ } = s in
  let set_order x y z = State.Order_state.update x (fun _ -> Some y) z in
  let set_interim x y z = State.Asset_owner.update x (fun _ -> Some y) z in
  (* When a user has their order partially filled, we must also destroy
   * their previous order and make a new one with the remainder that
   * they want to fill, so that it can be addressable later. *)
  let l_desired_unfilled = commit_left_inline_amt_unfilled c in
  let r_desired_unfilled = commit_right_inline_amt_unfilled c in
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

and apply_commit s = let open Applicative in function
  | Commit_inline o -> apply_inline_commit s o
  | Commit_onchain _ -> failwith "TODO"

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
