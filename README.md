
# Superposition Passport

Superposition Passport is a UTXO-based system of spending signatures given by the Matching
engine, which are then provided on-chain if the constraints are validated to the Solver
engine.

The conversion taking place internally is the conversion from the Applicative from in
`src/applicative.rs`, to `src/state_machine.rs`, with the application taking place using a
local conversion dependent on the state to the local contract. Following the conversion,
the state is converted to a local type that is converted to the Emissions type, which is
able to be bundled and emitted.

The applicative module exports a function that can do signature verification and check if
the state transition function is valid with the validate function. A stateful conversion
using the methods attached to the storage type is needed to convert the `CreateBalance` in
the applicative form conversion to `StateBalance`.

## Why?

A few reasons, namely:

1. Immediate offramping using only signatures to any chain

2. Easy explicit parallelism

3. Easy privacy later using the same approach with inclusion proofs

4. Super affordable compression that can be scaled to include a optimistic approach

5. Simple once you understand

## Diagram

The actual behaviour of the state transition looks like this:

![Diagram of the state transition](diagrams/state-transition.svg)

Emissions are consumed by the contract to produce side effects related to balances.
Storage is the contract itself. The Applicative type is provided to the contract to do the
conversion internally to the StateMachine. Conversions should only happen inside the
Solver contract. The applicative form conversion follows the following diagram:

![Diagram of the form conversion](diagrams/applicative-and-state-machine.svg)

With a conversion taking place from the applicative form by the solver to the internal
form, using the state inside the contract to support the conversion. It does the
conversion by looking at remaining balances after matching the applicative form. During
the Applicative form, the signatures are verified, and the state transition is validated.

But maybe it's better understood informally:

`storage.rs` is used with `state_machine.rs` to convert, then eventually consume, the
`applicative.rs` type, like this:

```mermaid
flowchart LR
    Validate["validate()"]
    --> Convert["convert()"]
    --> Apply["apply()"]
```

Which internally looks like the following:

```mermaid
flowchart TD
    Entry[Entrypoint]
    subgraph Validate["validate()"]
        Entry
        -->|"solve() . Function entrypoint."| Applicative
        -->|"validate() . Checks the signature of the Applicative blob."| Crypto
    end
    subgraph Convert["convert()"]
        subgraph StatefulStateMachine
            Storage -->|Provides the function| StateMachine
        end
        Applicative
        -->|"convert() . Converts Applicative to StateMachine"| StateMachine
    end
    subgraph Apply["apply()"]
        StateMachine -->|"apply() . Creates state"| Emissions
        Emissions -->|Given to entrypoint/emitted as logs| Entry
    end
```

So, solve is used as the entrypoint, which then kicks off validation of the signatures in
the applicative form, which then converts to the state machine form, which is then
persisted as state after conversion internally.

Each Balance is the creation of a snowflake for a user. It must be the timestamp it was
made from the user's point of view, and their address, hashed. The Solver contract
maintains an idea of the state of the Snowflake.

## Identifying the balances and their created derivatives

Identification is done using a Snowflake-like system of taking the user's timestamp and
their address, then keccak hashing it, then using the number as the snowflake as a
identifier.

When a recursively created derivative of a Balance creates a new Balance, a snowflake is
made. From the source:

```rust
/// Balances are identifiable in their descended form using the
/// concatenation of the previous hash, the timestamp of the change, and
/// the nonce here. keccak256(previous . nonce . timestamp).
#[repr(C)]
pub enum SnowflakeNonce {
    CreateBalance,
    SplitBalanceExcess,
    CommitFulfilledLeft,
    CommitFulfilledRight,
    CommisExcessLeft,
    CommitExcessRight,
    JoinBalance,
}
```

So, a create balance would have the nonce of 0, and be created using `keccak256(address .
0 . millisecond timestamp)`. A split balance would have 1, and be of the form
`keccak256(previous hash . 1 . excess amount)` and so on.

## User stories

The following user stories explain how to do recursive use of the applicative type and
it's translation to the state machine type. Some of the field arguments are elided and not
used in the below examples.

### Balance forking

#### Just the balance

Ivan goes to make a trade from ARB to OP. He asks for a price of $10, and has 5 ARB which
he wants to swap to OP, but last second he'll change his mind and only use 3 ARB. ARB and
OP are priced the same. Ivan creates the following Applicative structure:

	(Balance
	 <ivan sig>
	 (balance 'ARB 55244 5 'Ivan 1751349713))

This would be converted to this structure:

	(CreateBalance 'Ivan 'ARB 55244 5)

This results in a Snowflake of
`keccak256(abi.encodePacked(0x6221a9c005f6e47eb398fd867784cacfdcfff4e7,
uint128(1751349827449)))`. The result for the Solver contract to use to identify the state
of the Balance is `0x1177859b6194535e9e420abf7509a1dc4baee217eeae881188e09b4dfa54bc1f`.

#### Balance and orders

Ivan goes to place his newly created Balance on the market:

```scheme
(Order
 (order 3 'OP 55244 3)
 <ivan sig>
 (Balance
  <ivan sig>
  (balance 'ARB 55244 5 'Ivan 1751349713)))
```

This would be converted by the Solver to this structure during the `convert()` stage:

```scheme
(OrderOrigin
 (OrderOrigin.Single
  (OrderCreated 'OP 55244 3
   (SplitBalanceSpendable
    (CreateBalance 'Ivan 'ARB 55244 5)		; This makes up the input.
    (CreateBalance 'Ivan 'ARB 55244 3)		; This is the spendable output from this.
    (CreateBalance 'Ivan 'ARB 55244 2)))))	; This is the amount that could be reused.
```

In this situation, a new identifier would be made for the excess amount, which could be
reused to create a new balance like so. This would have the snowflake of
`keccak256(abi.encodePacked(bytes32(0x1177859b6194535e9e420abf7509a1dc4baee217eeae881188e09b4dfa54bc1f),
uint8(1), uint256(2)))`, aka
`0x2a48fd44c873f293067f14a14497ab3a7d54a6f4eef83d945ef277e1ad7d40f8`:

```scheme
(CreateBalanceOrigin
 (SplitBalanceSpendExcess
  (SplitBalance
   (CreateBalance 'Ivan 'ARB 55244 5)
   (CreateBalance 'Ivan 'ARB 55244 3)
   (CreateBalance 'Ivan 'ARB 55244 2))))	; This amount constitutes the balance of the CreateBalance here.
```

The solver knows the amount to fork off by keeping in mind the amounts available to be
spent by the Balance that were split in the past and being permissive, assuming that the
matching engine knows the correct amounts that can be spent (though it will check the
balances available to it assuming it has enough from what's committed in the past). This
translates into the matching engine knowing how much is available to be consumed.

#### Commitments (matching orders)

Knowing that Ivan has opted to spend his 3 ARB and the price is 1 to one, Eli has gone to
be the counterparty on his trade at a $1 price, but supplying 5 OP. Eli
constructs a Balance like the following:

```scheme
(Balance
 <eli sig>
 (balance 'OP 55244 5 'Eli 1751353972))
```

This would be converted to this structure for Eli:

```scheme
(CreateBalance 'Eli 'OP 55244 5)
```

Which he then wraps inside a Order:

```scheme
(Order (order 5 'ARB 55244 5)
 <eli sig>
 (Balance
 <eli sig>
 (balance 'OP 55244 5 'Eli 1751353972)))
```

Which is then translated to this type by the Solver:

```scheme
(OrderCreated 'ARB 55244 5
 (CreateBalance 'Eli 'OP 55244 5))
```

The solver sees that Ivan's order can be partially filled, and it creates the following
applicative structure from the two orders:

```scheme
(Commit
 <matcher sig>
 (Order (order 5 'ARB 55244 5)
  <eli sig>
  (Balance
   <eli sig>
   (balance 'OP 55244 5 'Eli 1751353972)))
 (Order (order 3 'OP 55244 3)
  <ivan sig>
  (Balance
   <ivan sig>
   (balance 'ARB 55244 5 'Ivan 1751349713))))
```

This states "Eli wants to exchange his 5 ARB for 5 OP, we can't fill it completely, but
the Matcher thinks this is the best outcome we can give Eli and Ivan right now based on
the orderbook". It's translated literally by the Solver to the state machine type:

```scheme
(Commit
 (OrderCreated 'ARB 55244 5				; The first balance that was filled (Eli).
   (CreateBalance 'Eli 'OP 55244 5 1751353972))
 (OrderCreated 'OP 55244 3				; This is Ivan's 3 OP he spent.
  (SplitBalanceSpendable
   (CreateBalance 'Ivan 'ARB 55244 5 1751349713)
   (CreateBalance 'Ivan 'ARB 55244 3 1751349713)
   (CreateBalance 'Ivan 'ARB 55244 2 1751349713)))
 (CreateBalance 'Eli 'ARB 55244 3 1751355318)	; This is Eli's filled balance.
 (CreateBalance 'Ivan 'OP 55244 3 1751355318)	; This is Ivan's filled balance.
 (Some											; This is Eli's excess order creation.
  (OrderCreated 'ARB 55244 2
   (StateBalance (CreateBalance 'Eli 'OP 55244 2 1751353972))))
 (Some											; This is Ivan's excess order creation.
  (OrderCreated 'OP 55244 2
   (StateBalance (CreateBalance 'Ivan 'OP 55244 2 1751353972)))))
```

#### Reusing the match result of the order

Eli can begin the cycle anew by taking the result of his order commitment by taking the
amount that wasn't immediately rolled into a new order by using it to construct a Balance,
then converting it to another order to buy some ETH, assuming ETH is worth $1, like so:

```scheme
(Order (order 3 'ETH 55244 2)
 <eli sig>
 (CommitLeftFilledToBalance	; Eli is using the amount that was filled and is now just a balance.
  (Commit
   <matcher sig>
   (Order (order 5 'ARB 55244 5)
    <eli sig>
    (Balance
     <eli sig>
     (balance 'OP 55244 5 'Eli 1751353972)))
   (Order (order 3 'OP 55244 3)
    <ivan sig>
    (Balance
     <ivan sig>
     (balance 'ARB 55244 5 'Ivan 1751349713))))))
```

This is converted to the state machine form like this by the Solver. The state machine has
another key difference with the applicative type in that it must always do a conversion to
a local balance instead of using ephereal storage somewhere. Since this is done by the
system during a local conversion, we can keep the type conversion here simple, so that the
applicative type must use an intermediate conversion to get the equivalent of a split
balance.

```scheme
(CommitSpendableLeft	; Eli uses the amount that was filled and converted to ARB.
 (Commit
  (OrderCreated 'ARB 55244 5
    (CreateBalance 'Eli 'OP 55244 5 1751353972))
  (OrderCreated 'OP 55244 3
   (SplitBalanceSpendable
    (CreateBalance 'Ivan 'ARB 55244 5 1751349713)
    (CreateBalance 'Ivan 'ARB 55244 3 1751349713)
    (CreateBalance 'Ivan 'ARB 55244 2 1751349713)))
  (CreateBalance 'Eli 'ARB 55244 3 1751355318)
  (CreateBalance 'Ivan 'OP 55244 3 1751355318)
  (Some
   (OrderCreated 'ARB 55244 2
    (StateBalance (CreateBalance 'Eli 'OP 55244 2 1751353972))))
  (Some
   (OrderCreated 'OP 55244 2
    (StateBalance (CreateBalance 'Ivan 'OP 55244 2 1751353972)))))
 (CreateBalance 'Eli 'ETH 55244 3 1751358901))	; This is the result from the leftover amount here.
```

#### Taking the results of the excessive amount and withdrawing

Eli can withdraw the amount that wasn't able to be matched by cancelling the order that
was created transitively from the Commit using the Cancel operation, then containing
that in the withdrawal operation.

```scheme
(Withdraw
 <matcher sig>
 <eli sig>
 (Cancel
  <matcher sig>
  <eli sig>
   (Order (order 3 'ETH 55244 2)
    <eli sig>
    (CommitLeftFilledToBalance
     (Commit
      <matcher sig>
      (Order (order 5 'ARB 55244 5)
       <eli sig>
       (Balance
        <eli sig>
        (balance 'OP 55244 5 'Eli 1751353972)))
      (Order (order 3 'OP 55244 3)
       <ivan sig>
       (Balance
        <ivan sig>
        (balance 'ARB 55244 5 'Ivan 1751349713))))))))
```

At which point he could supply this to the contract, for it to process his withdrawal.

Q. Why do we need a signature from the solver for the operation involving the withdrawing of balances?

Any operation that could split the balance in a stateful way, for example, an order being
matched with the Commit operation, Cancel, or Withdrawal, all depend on the balance not
being spent. Since signatures are ephmereal, there needs to be a chokepoint that ensures
that no double spending of balances is taking place.
