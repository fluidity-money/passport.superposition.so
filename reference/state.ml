module Orders_key = struct
  type t = {
    desired_asset : string;
    owner : string;
    desired_amt : int;
    from_asset : string;
  }
  [@@deriving show, ord, eq]
end

module Order_state = Map.Make (struct
  type t = Orders_key.t

  let compare = Orders_key.compare
end)

module Asset_owner_key = struct
  type t = { asset : string; owner : string } [@@deriving show, ord, eq]
end

module Asset_owner = Map.Make (struct
  type t = Asset_owner_key.t
  let compare = Asset_owner_key.compare
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
    | Balance_inline (`Bal_inline { bal_owner; bal_asset; bal_amt; _ }) ->
        {
          acc with
          withdrawable =
            Asset_owner.update
              Asset_owner_key.{ asset = bal_asset; owner = bal_owner }
              (function Some v -> Some (v + bal_amt) | None -> Some bal_amt)
              withdrawable;
        }
    | Balance_onchain (`Bal_onchain _) -> failwith "TODO"
    | Commit_left_filled_to_bal f | Commit_right_filled_to_bal f ->
        loop_commit acc f
    | Join (l, r) -> loop_bal (loop_bal acc l) r
    | Cancel o -> loop_order acc o
    | _ -> assert false
  and loop_commit acc = function
    | Commit_inline (`Commit_inline (_, l, r)) -> loop_order (loop_order acc l) r
    | Commit_onchain _ -> failwith "TODO"
    | _ -> assert false
  and loop_order acc = function
    | Order_inline (`Order_inline { ord_from; _ }) -> loop_bal acc ord_from
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
