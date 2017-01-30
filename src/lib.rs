#![feature(lookup_host)]
#![feature(sockaddr_checker)]
#![feature(libc)]
#![feature(associated_consts)]
#![feature(plugin)]
#![cfg_attr(test, plugin(quickcheck_macros))]

#[macro_use]
extern crate lazy_static;
extern crate byteorder;
extern crate rustc_serialize;
extern crate regex;
extern crate time;
#[macro_use]
extern crate log;

#[cfg(test)]
extern crate quickcheck;
extern crate clap;

pub mod app;
pub mod machine;
pub mod util;
pub mod flaker;
pub mod flake;
pub mod error;

