#![feature(slice_patterns)]
#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(conservative_impl_trait)]

extern crate llvm_sys as llvm;
#[macro_use]
extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;

pub mod math;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
