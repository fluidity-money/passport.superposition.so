(** The actual state machine of the application. A mixture of a GADT and a row
    level polymorphic type is used here to specialise the functions that work
    with these types. *)

type hash = string

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

and order_args = {
  ord_desired_asset : string;
  ord_desired_amt : int;
  ord_from : bal t;
}

and order_inline = [ `Order_inline of order_args ]
and order_onchain = [ `Order_onchain of string ]
and commit_inline = [ `Commit_inline of int * order t * order t ]
and commit_onchain = [ `Commit_onchain of string ]

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
    (`Order_inline { ord_desired_asset; ord_desired_amt; ord_from }) =
  Format.fprintf fmt
    "Order { ord_desired_asset = %s ; ord_desired_amt = %d ; ord_from = (%a)"
    ord_desired_asset ord_desired_amt pp_bal ord_from

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
