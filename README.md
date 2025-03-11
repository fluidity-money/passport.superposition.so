
# Superposition Passport

Superposition Passport is a smart account controllable with a secp256r1 signature derived
in the browser. It has two methods, `spend`, and `delegate`.

Spend calls a function after verifying the secp256r1 signature. It calls the calldata
given against the contract given, and then finally verifies the balances of the account to
see if the desired amount was spent, and if the desired amount of the other token is in
the owner of the smart account's account.

Superposition Passport is implemented using Stylus, with a custom encoding and decoding
package of Borsh, lz, Serde.

```mermaid
flowchart LR
    Spender((Spender))
    -->|Creates signature in browser using secp256r1 of call to contract with "requirements".| SmartAccount
```
