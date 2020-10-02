#![feature(map_first_last)]
#![feature(negative_impls)]

pub mod collections;
pub mod local;
pub mod span;
pub mod trace;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
