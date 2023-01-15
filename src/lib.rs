#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod hat;
pub use app::GiftExchangeApp;
use hat::Person;

fn valid_pair(a: &Person, b: &Person) -> bool {
    a.group != b.group
}
