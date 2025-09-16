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

impl TryFrom<u8> for Facet {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Facet::UserSolver),
            1 => Ok(Facet::UserSetter),
            2 => Ok(Facet::UserAdmin),
            3 => Ok(Facet::ReentrantVault),
            _ => Err(()),
        }
    }
}
