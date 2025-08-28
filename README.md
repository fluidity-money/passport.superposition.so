
# Superposition Passport

Superposition Passport is a UTXO-based system of spending signatures given by the Matching
engine, which are then provided on-chain if the constraints are validated to the Solver
engine.

The application processes the type in `src/applicative.rs`, after a validation step. It
then confirms local balances, and gradually moves balances in different buckets
denominated by the hash of each step in the application.

## Why?

A few reasons, namely:

1. Immediate offramping using only signatures to any chain

2. Easy explicit reuse of previously submitted parts of the application

3. Easy privacy later using the same approach with inclusion proofs

4. Super affordable compression that can be scaled to include a optimistic approach

5. Simple once you understand

## Diagram
