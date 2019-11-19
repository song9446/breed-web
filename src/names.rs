use rand::thread_rng;
use rand::seq::SliceRandom;
//use std::fs;

//const NAME_FILE: &str = "words_alpha.txt";
lazy_static::lazy_static! {
pub  static ref NAMES: Vec<&'static str> = include_str!("words_alpha.txt")
    .split('\n')
    .collect();
}
pub fn gen() -> String {
    NAMES.choose(&mut thread_rng()).unwrap().to_string() + NAMES.choose(&mut thread_rng()).unwrap()
}
