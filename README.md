
# Superposition Passport

Superposition Passport is a smart account controllable with a ed25519 signature derived
in the browser. It has two methods, `spend`, and `delegate`.

Spend calls a function after verifying the ed25519 signature. It calls the calldata
given with the contract specified, and then finally verifies the balances of the account to
see if the desired amount was spent, and if the desired amount of the other token is in
the owner of the smart account's account.

Superposition Passport is implemented using Stylus, with a custom encoding and decoding
package of Borsh and lzss.

Accounts are created by either manually setting them using the set address operation, or
by calling match for the first time if an account has been unset, but only if a permit
function is included in the batch.
