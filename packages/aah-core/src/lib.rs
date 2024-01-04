#![feature(associated_type_defaults)]
#![feature(path_file_prefix)]

pub mod adb;
pub mod config;
pub mod controller;
pub mod task;
pub mod vision;

pub fn add(left: usize, right: usize) -> usize {
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
