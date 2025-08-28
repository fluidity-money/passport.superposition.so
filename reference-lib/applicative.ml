(** The actual state machine of the application. A mixture of a GADT and a row
    level polymorphic type is used here to specialise the functions that work
    with these types. *)

type hash = int [@@deriving qcheck2]

type bal_args = {
  bal_ms_ts : int;
  bal_owner : string;
  bal_asset : string;
  bal_amt : int;
}

type bal_inline = [ `Bal_inline of bal_args ]
type bal_onchain = [ `Bal_onchain of hash ]

type bal
and commit
and order
and withdrawn
and reusable

and order_args = {
  ord_desired_asset : string;
  ord_desired_amt : int;
  ord_from : bal t;
  ord_max_pol_fee : int;
  ord_partial_fill_okay : bool;
}

and order_inline = [ `Order_inline of order_args ]
and order_onchain = [ `Order_onchain of hash ]
and commit_inline = [ `Commit_inline of int * order t * order t ]
and commit_onchain = [ `Commit_onchain of hash ]

and _ t =
  | Balance_inline : bal_inline -> bal t
  | Balance_onchain : bal_onchain -> bal t
  | Order_inline : order_inline -> order t
  | Order_onchain : order_onchain -> order t
  | Commit_inline : commit_inline -> commit t
  | Commit_onchain : commit_onchain -> commit t
  | Commit_left_filled_to_bal : commit t -> bal t
  | Commit_right_filled_to_bal : commit t -> bal t
  | Commit_left_excess_to_order : commit t -> order t
  | Commit_right_excess_to_order : commit t -> order t
  | Withdraw : bal t -> withdrawn t
  | Cancel : order t -> bal t
  | Join : bal t * bal t -> bal t
  | Reuse_for_filling : bal t -> reusable t

let rec pp_bal_inline fmt
    (`Bal_inline { bal_ms_ts; bal_owner; bal_asset; bal_amt }) =
  Format.fprintf fmt
    "Balance_inline { bal_ms_ts = %d;  bal_owner = %s; bal_asset = %s ; \
     bal_amt = %d }"
    bal_ms_ts bal_owner bal_asset bal_amt

and pp_bal_onchain _ (`Bal_onchain _) = failwith "TODO"

and pp_bal fmt = function
  | Balance_inline (`Bal_inline _ as b) -> pp_bal_inline fmt b
  | Balance_onchain (`Bal_onchain _ as b) -> pp_bal_onchain fmt b
  | Commit_left_filled_to_bal c ->
      Format.fprintf fmt "Commit_left_filled_to_bal { commit = %a }" pp_commit c
  | Commit_right_filled_to_bal c ->
      Format.fprintf fmt "Commit_right_filled_to_bal { commit = %a }" pp_commit
        c
  | Cancel o -> Format.fprintf fmt "Cancel %a" pp_order o
  | Join (x, y) -> Format.fprintf fmt "Join (%a, %a)" pp_bal x pp_bal y

and pp_order_inline fmt
    (`Order_inline
       {
         ord_desired_asset;
         ord_desired_amt;
         ord_from;
         ord_max_pol_fee;
         ord_partial_fill_okay;
       }) =
  Format.fprintf fmt
    "Order { ord_desired_asset = %s ; ord_desired_amt = %d ; ord_max_pol_fee = \
     %d ; ord_partial_fill_okay = %b ; ord_from = (%a)"
    ord_desired_asset ord_desired_amt ord_max_pol_fee ord_partial_fill_okay
    pp_bal ord_from

and pp_order_onchain _ (`Order_onchain _) = failwith "TODO"

and pp_order fmt = function
  | Order_inline o -> pp_order_inline fmt o
  | Order_onchain o -> pp_order_onchain fmt o
  | Commit_left_excess_to_order c ->
      Format.fprintf fmt "Commit_left_excess_to_order (%a)" pp_commit c
  | Commit_right_excess_to_order c ->
      Format.fprintf fmt "Commit_right_excess_to_order (%a)" pp_commit c

and pp_commit fmt = function
  | Commit_inline c -> pp_commit_inline fmt c
  | Commit_onchain _ -> failwith "TODO"

and pp_commit_inline fmt (`Commit_inline (ts, l, r)) =
  Format.fprintf fmt "Commit (%d, %a, %a)" ts pp_order l pp_order r

and pp_withdraw fmt (Withdraw b) = Format.fprintf fmt "Withdraw (%a)" pp_bal b

module Easy = struct
  open Ppx_sexp_conv_lib.Conv
  let hash_of_sexp= int_of_sexp
  let sexp_of_hash = sexp_of_int

  type easy_balance =
    [ `Easy_bal_inline of int * string * string * int
    | `Easy_bal_onchain of hash
    | `Easy_commit_left_filled_to_balance of easy_commit
    | `Easy_commit_right_filled_to_balance of easy_commit
    | `Easy_cancel of easy_order
    | `Easy_join of easy_balance * easy_balance ]

  and easy_order =
    [ `Easy_order_inline of string * int * easy_balance * int * bool
    | `Easy_order_onchain of hash
    | `Easy_commit_left_excess_to_order of easy_commit
    | `Easy_commit_right_excess_to_order of easy_commit ]

  and easy_commit =
    [ `Easy_commit_inline of int * easy_order * easy_order
    | `Easy_commit_onchain of int ]

  and easy_withdraw = [ `Easy_withdraw of easy_balance ] [@@deriving qcheck2, sexp]

  let rec of_apply_inline_bal
      (`Bal_inline { bal_ms_ts; bal_owner; bal_asset; bal_amt }) =
    `Easy_bal_inline (bal_ms_ts, bal_owner, bal_asset, bal_amt)

  and of_apply_bal = function
    | Balance_inline b -> of_apply_inline_bal b
    | Balance_onchain (`Bal_onchain h) -> `Easy_bal_onchain h
    | Commit_left_filled_to_bal o ->
        `Easy_commit_left_filled_to_balance (of_apply_commit o)
    | Commit_right_filled_to_bal o ->
        `Easy_commit_right_filled_to_balance (of_apply_commit o)
    | Cancel o -> `Easy_cancel (of_apply_order o)
    | Join (l, r) -> `Easy_join (of_apply_bal l, of_apply_bal r)

  and of_apply_inline_order
      (`Order_inline
         {
           ord_desired_asset;
           ord_desired_amt;
           ord_from;
           ord_max_pol_fee;
           ord_partial_fill_okay;
         }) =
    `Easy_order_inline
      ( ord_desired_asset,
        ord_desired_amt,
        of_apply_bal ord_from,
        ord_max_pol_fee,
        ord_partial_fill_okay )

  and of_apply_order = function
    | Order_inline o -> of_apply_inline_order o
    | Order_onchain (`Order_onchain h) -> `Easy_order_onchain h
    | Commit_left_excess_to_order c ->
        `Easy_commit_left_excess_to_order (of_apply_commit c)
    | Commit_right_excess_to_order c ->
        `Easy_commit_right_excess_to_order (of_apply_commit c)

  and of_apply_inline_commit (`Commit_inline (ts, l, r)) =
    `Easy_commit_inline (ts, of_apply_order l, of_apply_order r)

  and of_apply_commit = function
    | Commit_inline c -> of_apply_inline_commit c
    | Commit_onchain (`Commit_onchain h) -> `Easy_commit_onchain h

  and of_apply_withdraw (Withdraw w) =
    `Easy_withdraw (of_apply_bal w)

  let rec to_apply_bal = function
    | `Easy_bal_inline (bal_ms_ts, bal_owner, bal_asset, bal_amt) ->
        Balance_inline
          (`Bal_inline { bal_ms_ts; bal_owner; bal_asset; bal_amt })
    | `Easy_bal_onchain h -> Balance_onchain (`Bal_onchain h)

  and to_apply_order = function
    | `Easy_order_inline
        ( ord_desired_asset,
          ord_desired_amt,
          from,
          ord_max_pol_fee,
          ord_partial_fill_okay ) ->
        Order_inline
          (`Order_inline
             {
               ord_desired_asset;
               ord_desired_amt;
               ord_from = to_apply_bal from;
               ord_max_pol_fee;
               ord_partial_fill_okay;
             })
    | `Easy_order_onchain h -> Order_onchain (`Order_onchain h)
    | `Easy_commit_left_excess_to_order h ->
        Commit_left_excess_to_order (to_apply_commit h)
    | `Easy_commit_right_excess_to_order h -> Commit_right_excess_to_order h

  and to_apply_commit = function
    | `Easy_commit_inline (ts, l, r) ->
        Commit_inline (`Commit_inline (ts, to_apply_order l, to_apply_order r))
    | `Easy_commit_onchain h -> Commit_onchain (`Commit_onchain h)
end
