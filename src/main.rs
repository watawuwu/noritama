extern crate noritama;

use noritama::app::App;
use noritama::app::error::Error;

use std::io::Write;
use std::process::exit;

const SUCCESS_EXIT_CODE: &'static i32 = &0;
const FAILURE_EXIT_CODE: &'static i32 = &1;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

pub fn abort(err: Error, msg: &str, err_code: &i32) -> ! {
    println_stderr!("{} {}", msg, err);
    exit(*err_code)
}

fn main() {
    let id = match App::run() {
        Ok(id) => id,
        Err(e) => abort(e, "Failed!!", FAILURE_EXIT_CODE),
    };
    println!("{}", id);

    exit(*SUCCESS_EXIT_CODE);
}
