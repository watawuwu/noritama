#![feature(test)]

extern crate noritama;
extern crate test;

use noritama::flaker::*;

use test::Bencher;

fn flaker() -> Flaker {
    Flaker::new().unwrap()
}

#[bench]
fn gen(b: &mut Bencher) {
    let c: Flaker = flaker();
    b.iter(|| {
        c.id().unwrap()
    });
}

