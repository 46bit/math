#![feature(alloc_system)]
#![feature(slice_patterns)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(conservative_impl_trait)]

extern crate alloc_system;
extern crate libc;
extern crate llvm_sys as llvm;
#[macro_use]
extern crate nom;
extern crate quickcheck;
extern crate rand;

pub mod math;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
