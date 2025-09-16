#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Facet {
    // User facets. First loaded using compression.
    UserSolver = 0,
    UserSetter = 1,
    UserAdmin = 2,
    // Reentrant facets. Accessible using a reentrant call and some storage
    // use. Compression does not take place here.
    ReentrantVault = 3,
}
