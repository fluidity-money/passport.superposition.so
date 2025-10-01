
# Recipes

## Unsolved recipes (.upp)

Unsolved recipes are of the kind:

	(Balance Alex
	 ((asset aeff361bab3708fa59f2972fd4a3900ba33ce821)
	  (chain 0)
	  (amount 123)
	  (ms_timestamp 1759221563622)))

This form lacks the hash, and the signature. This form is used alongside the solve
function to create the solved recipe form, that can be converted to calldata. Examples
live in this directory. The signer's string identifier is included in each step, and is
used by the offline tool to use the signer to drop in the signatures where appropriate. If
a signer is not found, the program will fail. When the conversion happens, it should use
the signing key.

## Solved recipes (.spp)

This form contains the signature, and the hash. This form can be converted to calldata.

	(Solve (0)	# These are the account offsets.
	(Balance
	 (0 128f8c7a2b03606b605484b430d9dc9c7dfbacdb713978b0f80667b5921e0dda4c967509e0b33ec84f28d9613cf128a240da8398fd74fd06ba8df8fe95b0dc00)
	 ((asset aeff361bab3708fa59f2972fd4a3900ba33ce821)
	  (chain 0)
	  (amount 123)
	  (ms_timestamp 1759221563622))))
