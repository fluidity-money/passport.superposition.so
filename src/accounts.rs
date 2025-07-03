use ed25519_dalek::VerifyingKey;

/// Applicative signature map for a signer verifying key and their place
/// in the map.
#[derive(Debug, PartialEq, Eq)]
pub struct Accounts {
    pub solver: VerifyingKey,
}

impl Accounts {
    pub fn find<'a>(&self, _id: usize) -> Option<&'a VerifyingKey> {
        None
    }
}

impl Default for Accounts {
    fn default() -> Self {
         Accounts{ solver: VerifyingKey::default() }
    }
}