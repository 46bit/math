#![feature(slice_patterns)]
#![feature(box_syntax)]
#![feature(box_patterns)]

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
