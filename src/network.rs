
#[derive(Debug, Clone, Copy)]
pub enum Network {
     CUSTOM,
     TESTNET,
     MAINNET,
}

#[cfg(any(
    all(feature = "network-mainnet", feature = "network-testnet"),
    all(feature = "network-mainnet", feature = "network-custom"),
    all(feature = "network-testnet", feature = "network-custom")
))]
compile_error!("multiple networks enabled");
