//! WASI random_get using the getrandom crate.

pub fn random_get(buf: &mut [u8]) -> u32 {
    match getrandom::getrandom(buf) {
        Ok(()) => 0,
        Err(_) => 29, // EIO
    }
}
