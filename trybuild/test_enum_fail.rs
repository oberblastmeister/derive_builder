use derive_builder::Builder;

#[derive(Builder)]
pub enum Command {
    One,
    Two,
    Three,
}

fn main() {}
