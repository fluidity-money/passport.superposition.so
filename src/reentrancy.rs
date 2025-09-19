// We have a weird language for talking about reentrancy here. We refer
// to arguments to the reentrant program as "articles" that are known to
// the child process. This method lets us avoid the overhead of decoding
// the arguments to the function, instead preferring to make several
// TSTORE calls. Invoking the begin_reentrancy function sets the carnary
// and calls the facet given.

use crate::{ops::OpVault};

pub const ARTICLES_VAULT: [[u8; 32]; 1] = [[0u8; 32]];

pub fn begin_vault_reentrancy(_op: OpVault, _articles: &[([u8; 32], [u8; 32])]) {}

pub fn end_reentrancy() {}
