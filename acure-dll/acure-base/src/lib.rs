use std::ptr::null_mut;

pub type pAcure = *mut Acure;

#[repr(C)]
pub struct Acure {
    inner: acure::Acure
}

#[no_mangle]
pub extern "C" fn CreateAcure() -> pAcure {
    let p_acure = Acure {
        inner: acure::Acure::new()
    };
    null_mut()
}

#[no_mangle]
pub extern "C" fn greeting() {
    println!("Hello, {}", "Rust");
}