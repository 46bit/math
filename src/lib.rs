#![feature(slice_patterns)]

#[macro_use]
extern crate nom;

pub mod math;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
