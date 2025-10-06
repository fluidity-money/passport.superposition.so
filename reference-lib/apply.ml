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

and order_inline_desired_amt Applicative.(`Order_inline { ord_desired_amt; _ })
    =
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
  | Order_inline o -> order_inline_desired_amt o
  | Order_onchain o -> ord_onchain_desired_amt o
  | Commit_left_excess_to_order c -> commit_left_amt_unfilled c
  | Commit_right_excess_to_order c -> commit_right_amt_unfilled c

and order_inline_desired_asset
    Applicative.(`Order_inline { ord_desired_asset; _ }) =
  ord_desired_asset

and ord_desired_asset =
  let open Applicative in
  function
  | Order_inline o -> order_inline_desired_asset o
  | Order_onchain _ -> failwith "TODO"
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
  try ord_desired_amt r - ord_amt l with Checked_sub (_, _, _) -> 0

and commit_right_inline_amt_unfilled (`Commit_inline (_, l, r)) =
  try ord_desired_amt l - ord_amt r with Checked_sub (_, _, _) -> 0

and apply_bal_inline (State.{ withdrawable; interim; _ } as s)
    Applicative.(`Bal_inline { bal_owner; bal_asset; bal_amt; _ } as b) =
  if bal_amt <= 0 then invalid_arg "Balance is zero";
  State.
    {
      s with
      interim =
        Order_state.update
          (Orders_key.of_apply_inline_bal bal_owner b)
          (function Some v -> Some (v + bal_amt) | None -> Some bal_amt)
          interim;
      withdrawable =
        Asset_owner.update
          State.Asset_owner_key.{ asset = bal_asset; owner = bal_owner }
          (function
            | Some withdrawable when bal_amt > withdrawable ->
                invalid_arg "Not enough withdrawable"
            | Some withdrawable -> Some (withdrawable - bal_amt)
            | None -> invalid_arg "No withdrawable amount")
          withdrawable;
    }

and apply_cancel s (Applicative.Cancel o as c) =
  let amt = ord_amt o in
  let owner = ord_owner o in
  let s = apply_order s o in
  let State.{ orders; interim; _ } = s in
  let k = State.Orders_key.of_apply_ord owner o in
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
                        (%a), tried to take amount %d, only have %d. Storage: \
                        %a"
                       State.Orders_key.pp k amt v State.pp s)
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
          Order_state.update
            (Orders_key.of_apply_bal owner c)
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

and apply_inline_order s Applicative.(`Order_inline { ord_from; _ } as o) =
  let from_asset = order_inline_asset o in
  let owner = order_inline_owner o in
  let amt = order_inline_amt o in
  let desired_asset = order_inline_desired_asset o in
  if from_asset = desired_asset then
    invalid_arg "Same asset desired as supplied";
  let s = apply_bal s ord_from in
  let State.{ orders; interim; _ } = s in
  State.
    {
      s with
      orders =
        Order_state.update
          (Orders_key.of_apply_inline_ord owner o)
          (function Some v -> Some (v + amt) | None -> Some amt)
          orders;
      interim =
        Order_state.update
          (Orders_key.of_apply_bal owner ord_from)
          (function
            | Some v when amt > v ->
                invalid_arg
                  (Format.sprintf
                     "Not enough interim for order, have %d, want %d" v amt)
            | Some v -> Some (v - amt)
            | None -> invalid_arg "no balance for interim order")
          interim;
    }

and apply_order s =
  let open Applicative in
  function
  | Order_inline c -> apply_inline_order s c
  | Order_onchain (`Order_onchain _) -> failwith "TODO"
  | Commit_left_excess_to_order c | Commit_right_excess_to_order c ->
      apply_commit s c

and apply_inline_commit s (`Commit_inline (_, l, r) as c) =
  let l_desired_asset = ord_desired_asset l in
  let r_desired_asset = ord_desired_asset r in
  if not (String.equal (ord_asset r) l_desired_asset) then
    invalid_arg
      (Format.sprintf
         "Incorrect desired left asset: order asset right: %s, left desired \
          asset: %s"
         (ord_asset r) l_desired_asset);
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
  let set_order_key x y z = State.Order_state.update x (fun _ -> Some y) z in
  (* When a user has their order partially filled, we must also destroy
     their previous order and make a new one with the remainder that
     they want to fill, so that it can be addressable later. *)
  let l_prev_key = State.Orders_key.of_apply_ord l_owner l in
  let r_prev_key = State.Orders_key.of_apply_ord r_owner r in
  let commit_inline = Applicative.Commit_inline c in
  let l_unfilled_key =
    State.Orders_key.of_apply_ord l_owner
      (Commit_left_excess_to_order commit_inline)
  in
  let r_unfilled_key =
    Applicative.(
      State.Orders_key.of_apply_ord r_owner
        (Commit_right_excess_to_order commit_inline))
  in
  let l_filled_key =
    State.Orders_key.of_apply_bal l_owner
      (Commit_left_filled_to_bal commit_inline)
  in
  let r_filled_key =
    State.Orders_key.of_apply_bal r_owner
      (Commit_right_filled_to_bal commit_inline)
  in
  State.
    {
      s with
      interim =
        set_order_key l_filled_key l_filled
          (set_order_key r_filled_key r_filled interim);
      orders =
        set_order_key l_unfilled_key l_unfilled
          (set_order_key r_unfilled_key r_unfilled
             (set_order_key l_prev_key 0 (set_order_key r_prev_key 0 orders)));
    }

and apply_commit s =
  let open Applicative in
  function
  | Commit_inline o -> apply_inline_commit s o
  | Commit_onchain _ -> failwith "TODO"

and apply_withdraw s (Applicative.Withdraw b) =
  let owner = bal_owner b in
  let asset = bal_asset b in
  let amt = bal_amt b in
  let s = apply_bal s b in
  let State.{ interim; withdrawable; _ } = s in
  let interim_k = State.Orders_key.of_apply_bal owner b in
  State.
    {
      s with
      interim =
        State.Order_state.update interim_k
          (function
            | Some v when amt > v ->
                invalid_arg
                  (Format.asprintf
                     "Not enough interim state (%a), key (%a), needed: %d, \
                      only have %d"
                     State.pp s State.Orders_key.pp interim_k amt v)
            | Some v -> Some (v - amt)
            | None -> invalid_arg "No interim balance")
          interim;
      withdrawable =
        Asset_owner.update Asset_owner_key.{ asset ; owner }
          (function Some v -> Some (v + amt) | None -> Some amt)
          withdrawable;
    }
