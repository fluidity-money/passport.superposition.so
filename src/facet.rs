#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum Facet {
    Solver = 0,
    Setter = 1,
}
