extern crate ms3d;

use std::fs::File;
use ms3d::Model;

const BYTES: &[u8] = include_bytes!("POA.ms3d");

#[test]
fn test_reader() {
    Model::from_reader(File::open("tests/POA.ms3d").unwrap()).unwrap();
}

#[test]
fn test_slice() {
    Model::from_bytes(BYTES).unwrap();
}