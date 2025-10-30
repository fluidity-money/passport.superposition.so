
#[cfg(target_arch = "x86_64")]
mod accounts;
#[cfg(target_arch = "x86_64")]
mod convert;
#[cfg(target_arch = "x86_64")]
mod reader;
#[cfg(target_arch = "x86_64")]
mod unsolved;

#[cfg(target_arch = "x86_64")]
mod host;

#[cfg(target_arch = "x86_64")]
fn main() {
    host::entry()
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {}
