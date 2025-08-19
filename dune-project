(lang dune 3.19)

(name passport_superposition_so)

(generate_opam_files false)

(source
 (github fluidity-money/passport.superposition.so))

(authors "Fluidity Labs Team <alex@fluidity.money>")

(maintainers "Bayge <alex@fluidity.money>")

(license LICENSE)

(documentation https://url/to/documentation)

(package
 (allow_empty)
 (name passport_superposition_so)
 (synopsis "Reference implementation of Superposition Passport, with mocked storage.")
 (description "Superposition Passport (OCaml edition) is a mocked implementation of Superposition's in-house rollup engine, built using Arbitrum Stylus. It aggregates signatures and performs an on-chain coincidence of wants matching. It is inherently cross-chain using Hyperlane and Espresso, so it's implementation is mocked in this reference.")
 (depends ocaml ounit2 qcheck ppx_deriving)
 (tags
  (arbitrum web3 stylus rollup)))
