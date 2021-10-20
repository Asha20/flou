use std::convert::TryFrom;

use flou::{Flou, FlouError, Renderer, SvgRenderer};

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("error: missing input file");
        std::process::exit(1);
    });

    let input = std::fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("error reading file: {}", e);
        std::process::exit(1);
    });

    let flou = Flou::try_from(input.as_str()).unwrap_or_else(|e| {
        eprintln!("error parsing flou:");
        match e {
            FlouError::Parse(e) => {
                eprintln!("parse error:");
                eprintln!("{}", e);
            }
            FlouError::Logic(e) => {
                eprintln!("logic error: {:?}", e);
            }
        };
        std::process::exit(1);
    });

    println!("{}", SvgRenderer::default().render(&flou))
}
