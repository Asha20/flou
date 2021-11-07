use flou::{Flou, FlouError, LogicError, Renderer, ResolutionError, SvgRenderer};
use std::convert::TryFrom;
use std::fmt;
use std::io::{BufWriter, Write};
use std::{
    fs,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Input file; use "-" to read input from stdin.
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file; outputs to stdout if omitted.
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,

    /// Specifies the width and height of nodes in the grid (format: x,y).
    #[structopt(short = "n", long = "node", parse(try_from_str = parse_size))]
    node_size: Option<(i32, i32)>,

    /// Specifies the width and height of the grid gaps (format: x,y).
    #[structopt(short = "g", long = "gap", parse(try_from_str = parse_size))]
    grid_gap_size: Option<(i32, i32)>,
}

fn parse_size(src: &str) -> Result<(i32, i32), &'static str> {
    let tokens = src.split(',').collect::<Vec<_>>();
    if tokens.len() != 2 {
        return Err("Size should have format: \"x,y\"");
    }

    let x = tokens[0]
        .parse::<i32>()
        .map_err(|_| "Could not parse X coordinate")?;
    let y = tokens[1]
        .parse::<i32>()
        .map_err(|_| "Could not parse Y coordinate")?;

    if x < 0 || y < 0 {
        return Err("X and Y cannot be negative.");
    }

    Ok((x, y))
}

pub enum Error {
    InputOpen(io::Error),
    InputRead(io::Error),
    OutputOpen(io::Error),
    OutputWrite(io::Error),
    Parse(String),
}

pub fn run(opt: Opt) -> Result<(), Error> {
    let mut reader: Box<dyn BufRead> = if opt.input != PathBuf::from("-") {
        fs::File::open(opt.input)
            .map(|x| -> Box<dyn BufRead> { Box::new(BufReader::new(x)) })
            .map_err(Error::InputOpen)?
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    let mut writer: Box<dyn Write> = if let Some(filename) = opt.output {
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(filename)
            .map(|x| -> Box<dyn Write> { Box::new(BufWriter::new(x)) })
            .map_err(Error::OutputOpen)?
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    let mut input = String::new();
    reader
        .read_to_string(&mut input)
        .map_err(Error::InputRead)?;

    let flou = Flou::try_from(input.as_str()).map_err(|x| Error::Parse(flou_error_to_string(x)))?;

    let renderer = SvgRenderer::new(opt.node_size, opt.grid_gap_size);
    let output = renderer.render(&flou);

    write!(writer, "{}", output).map_err(Error::OutputWrite)?;

    Ok(())
}

fn flou_error_to_string(e: FlouError) -> String {
    match e {
        FlouError::Parse(e) => {
            format!("Error parsing Flou:\n\n{}", e)
        }
        FlouError::Logic(e) => {
            format!("Error in Flou logic:\n{}", logic_error_to_string(e))
        }
    }
}

fn logic_error_to_string(e: LogicError) -> String {
    match e {
        LogicError::DuplicateLabels(labels) => {
            let labels = print_map(labels, "\n", |label, locations| {
                let locations = print_sequence(locations, ", ", |x| x.to_string());
                format!("  - \"{}\" at: {}", label, locations)
            });

            format!("Some labels are used more than once:\n\n{}", labels)
        }
        LogicError::DuplicateDefinitions(ids) => {
            let ids = print_sequence(ids, "\n", |id| format!("  - \"{}\"", id));
            format!("Some identifiers have multiple definitions:\n\n{}", ids)
        }
        LogicError::DuplicateNodeAttributesInDefinitions(attrs) => {
            let attrs = print_map(attrs, "\n", |id, attrs| {
                let attrs = print_sequence(attrs, ", ", quote);
                format!("  - \"{}\" has duplicate(s): {}", id, attrs)
            });

            format!(
                "Some node definitions have duplicate attributes:\n\n{}",
                attrs
            )
        }
        LogicError::DuplicateNodeAttributesInGrid(attrs) => {
            let attrs = print_map(attrs, "\n", |id, attrs| {
                let attrs = print_sequence(attrs, ", ", quote);
                format!("  - Node at {} has duplicate(s): {}", id, attrs)
            });

            format!(
                "Some nodes declared in the grid have duplicate attributes:\n\n{}",
                attrs
            )
        }
        LogicError::DuplicateConnectionAttributesInDefinitions(attrs) => {
            let attrs = print_map(attrs, "\n", |id, index_map| {
                let indexes = print_map(index_map, "\n", |index, attrs| {
                    format!(
                        "    - At index {}: {}",
                        index,
                        print_sequence(attrs, ", ", quote)
                    )
                });

                format!("  - At definition \"{}\":\n{}", id, indexes)
            });

            format!(
                "Some connections in node definitions have duplicate attributes:\n\n{}",
                attrs
            )
        }
        LogicError::DuplicateConnectionAttributesInGrid(attrs) => {
            let attrs = print_map(attrs, "\n", |pos, index_map| {
                let index_map = print_map(index_map, "\n", |index, attrs| {
                    format!(
                        "    - For connection at index {}: {}",
                        index,
                        print_sequence(attrs, ", ", quote)
                    )
                });

                format!("  - At grid position {}:\n{}", pos, index_map)
            });

            format!(
                "Some connections declared in the grid have duplicate attributes:\n\n{}",
                attrs
            )
        }
        LogicError::InvalidDestination(errors) => {
            let errors = print_map(errors, "\n", |pos, index_map| {
                let index_map = print_map(index_map, "\n", |index, error| {
                    format!(
                        "    - For connection at index {}: {}",
                        index,
                        print_resolution_error(error)
                    )
                });

                format!("  - For node at grid position {}:\n{}", pos, index_map)
            });

            format!(
                "Could not resolve destination for some node's connections:\n\n{}",
                errors
            )
        }
    }
}

fn quote<T: fmt::Display>(item: T) -> String {
    format!("\"{}\"", item)
}

fn print_sequence<T: fmt::Display, I: IntoIterator<Item = T>>(
    seq: I,
    delimiter: &str,
    print: impl Fn(T) -> String,
) -> String {
    seq.into_iter()
        .map(print)
        .collect::<Vec<_>>()
        .join(delimiter)
}

fn print_map<K: fmt::Display, V, I: IntoIterator<Item = (K, V)>>(
    map: I,
    delimiter: &str,
    print: impl Fn(K, V) -> String,
) -> String {
    map.into_iter()
        .map(|(k, v)| print(k, v))
        .collect::<Vec<_>>()
        .join(delimiter)
}

fn print_resolution_error(e: ResolutionError) -> String {
    match e {
        ResolutionError::InvalidDirection(dir) => {
            format!("No destination found in direction: {}", dir)
        }
        ResolutionError::UnknownLabel(label) => format!("No destination with label: \"{}\"", label),
    }
}
