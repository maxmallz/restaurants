mod front_of_house;
pub use crate::front_of_house::hosting;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

pub fn eat_at_restaurant() {
    // Absolute path
    hosting::add_to_waitlist();
    front_of_house::hosting::add_to_waitlist();
    crate::front_of_house::hosting::add_to_waitlist();
}
