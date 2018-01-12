extern crate ms3d;

use std::fs::File;
use ms3d::Model;

#[test]
fn main() {
    Model::from_reader(File::open("tests/POA.ms3d").unwrap()).unwrap();
}
