
# Superposition Passport

Superposition Passport is a UTXO-based system of spending signatures given by the Matching
engine, which are then provided on-chain if the constraints are validated to the Solver
engine.

The application processes the type in `src/applicative.rs`, after a validation step. It
then confirms local balances, and gradually moves balances in different buckets
denominated by the hash of each step in the application.

## Diagram

### High level infrastructure diagram

```mermaid
flowchart TD
  subgraph Longtail["Matches orders together using an off-chain orderbook"]
    Orderbook --> Solver
  end

  subgraph Passport["Moves ownership of assets around on-chain"]
    StateMachine["State machine"]
    Validate --> StateMachine
  end

  Solver
  -->|Requests moving of liquidity| Vault
  -->|Can be used to offramp funds from passport| Validate
```

### Passport diagram

```mermaid
flowchart LR
  Applicative
  -->|The applicative form is validated| StateMachine[State machine]
  -->|The state machine form is applied to the storage| Application
```
