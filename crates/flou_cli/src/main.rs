use flou_cli::{run, Error, Opt};
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    run(opt).unwrap_or_else(|e| {
        match e {
            Error::InputOpen(e) => eprintln!("Could not open input file: {}", e),
            Error::InputRead(e) => eprintln!("Could not read input: {}", e),
            Error::OutputOpen(e) => eprintln!("Could not open output file: {}", e),
            Error::OutputWrite(e) => eprintln!("Could not write output: {}", e),
            Error::Parse(e) => eprintln!("{}", e),
        };

        std::process::exit(1);
    });
}
