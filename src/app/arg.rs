use clap;

const DECODE: &'static str = "decode";
const START_EPOCH: &'static str = "start-epoch";

#[derive(Debug)]
pub struct Arg {
    pub id: Option<String>,
    pub start_epoch: Option<String>,
}

impl Arg {
    pub fn read() -> Arg {

        let matches = clap::App::new("Noritama produces k-sorted ids.")
            .version("0.1")
            .author("watawuwu <watawuwu@gmail.com>")
            .about("Noritama produces k-sorted ids.")
            .arg(clap::Arg::with_name(DECODE)
                .short("d")
                .long(DECODE)
                .help("decoded hexed id")
                .takes_value(true))
            .arg(clap::Arg::with_name(START_EPOCH)
                .short("s")
                .long(START_EPOCH)
                .help("epoch used for internal timestamp")
                .takes_value(true))
            .get_matches();

        debug!("matchers{:?}", matches);

        let in_id = matches.value_of(DECODE);
        let in_start_epoch = matches.value_of(START_EPOCH);

        Arg {
            id: in_id.map(|id| id.to_string()),
            start_epoch: in_start_epoch.map(|s| s.to_string()),
        }
    }
}
