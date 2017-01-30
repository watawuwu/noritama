extern crate env_logger;

pub mod arg;
pub mod error;

use flaker::{Flakerable, Flaker};
use flake::Flakable;
use app::error::*;
use app::arg::Arg;


pub struct App;
impl App {
    pub fn run() -> Result<String, Error> {
        let _ = env_logger::init()?;
        let arg = Arg::read();

        debug!("arg: {:?}", arg);

        match arg.id {
            Some(hexed_id) => App::decode(&hexed_id, arg.start_epoch),
            None => App::gen(arg.start_epoch),
        }
    }

    fn flaker(maybe_start_epoch: Option<String>) -> Result<Flaker, Error> {
        let start_epoch: Result<Option<u64>, Error> = maybe_start_epoch.map_or(Ok(None), |s| {
            s.parse::<u64>()
                .map_err(|_| Error::InvalidArgError("Invalid start-epoch".to_string()))
                .map(|s| Some(s))
        });

        match start_epoch {
            Ok(Some(s)) => Ok(Flaker::new_with_epoch_timestamp(s, None)?),
            Ok(None) => Ok(Flaker::new()?),
            Err(err) => Err(err),
        }
    }

    fn decode(hexed_id: &str, maybe_start_epoch: Option<String>) -> Result<String, Error> {
        let flaker = Self::flaker(maybe_start_epoch)?;
        let flake = flaker.id_with_str(hexed_id)?;

        Ok(flake.to_string())
    }

    fn gen(maybe_start_epoch: Option<String>) -> Result<String, Error> {
        let flaker = Self::flaker(maybe_start_epoch)?;
        let flake = flaker.id()?;

        Ok(flake.hex())
    }
}
