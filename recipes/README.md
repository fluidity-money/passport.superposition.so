
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

	(Balance
	 (0
	  110fde122d8b0b25d9e6fa5c15763c96990d0dd33fdf79f73043904bb1541528c80dfea2541c396fa606156743750f249089f364f875879ef2c50b914a16c801)
	 ((asset 8ac1c7a5416e7e0342b532a9dc9d74c998e0e790)
	  (chain 225476647479317694062150620526444369903)
	  (amount 126508738668270503307039462831516350703)
	  (ms_timestamp 88720005195802133222596758088858595407)))

This form is a bit more involved than the other form.
