#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_unsafe)]
#![allow(unused_braces)]

// In the real would this should probably be in it's own crate to speed up compile times
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(unsafe { crate::add(2, 2) }, 4);
        assert_eq!(unsafe { crate::sub(10, 5) }, 5);
        assert_eq!(unsafe { crate::mul(10, 10) }, 100);
        assert_eq!(unsafe { crate::div(10, 2) }, 5);
    }
}
