#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;

mod importer;
#[cfg(test)]
mod test;

use clap::{App, Arg};
use importer::{clean_empty_dirs, move_and_sort, Config, Language};
use std::path::PathBuf;

fn main() {
    let config = parse_args();
    if !config.input.exists() {
        eprintln!("The source directory does not exist!");
    } else {
        move_and_sort(&config);
        if config.clean {
            clean_empty_dirs(config.input, config.verbose)
        }
    }
}

fn parse_args() -> Config {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Prints messages for successful imports"),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .short("n")
                .help("Only get the dates from the filenames"),
        )
        .arg(
            Arg::with_name("meta")
                .long("meta")
                .short("e")
                .help("Only get the dates from the metadata"),
        )
        .arg(
            Arg::with_name("clean")
                .short("c")
                .long("clean")
                .help("Remove empty directories from the input"),
        )
        .arg(
            Arg::with_name("limit")
                .short("l")
                .long("limit")
                .value_name("YEAR")
                .help("The oldest possible year")
                .takes_value(true)
                .default_value("1950")
                .validator(|v| {
                    v.parse::<i32>()
                        .map(|_| ())
                        .map_err(|_| "Must be a number".to_string())
                }),
        )
        .arg(
            Arg::with_name("year")
                .short("y")
                .long("year")
                .help("Add year to the names of the monthly directories"),
        )
        .arg(
            Arg::with_name("month")
                .short("m")
                .long("month")
                .help("Add month names to the monthly directories")
                .takes_value(true)
                .value_name("LANGUAGE")
                .possible_value("en")
                .possible_value("swe"),
        )
        .arg(
            Arg::with_name("flat")
                .short("f")
                .long("flat")
                .help("Flatten the directory structure (combine year and month)"),
        )
        .arg(
            Arg::with_name("input")
                .takes_value(true)
                .required(true)
                .value_name("INPUT")
                .help("The source directory"),
        )
        .arg(
            Arg::with_name("output")
                .takes_value(true)
                .required(true)
                .value_name("OUTPUT")
                .help("The destination directory"),
        )
        .get_matches();
    let meta = args.is_present("meta");
    let name = args.is_present("name");
    Config {
        input: PathBuf::from(args.value_of("input").unwrap()),
        output: PathBuf::from(args.value_of("output").unwrap()),
        verbose: args.is_present("verbose"),
        name: !meta || name,
        meta: !name || meta,
        clean: args.is_present("clean"),
        min_year: args
            .value_of("limit")
            .unwrap()
            .parse::<i32>()
            .expect("The year limit must be a number"),
        year: args.is_present("year"),
        month: if args.is_present("month") {
            match args.value_of("month").unwrap() {
                "en" => Language::English,
                "swe" => Language::Swedish,
                _ => panic!("Unknown language for month names"),
            }
        } else {
            Language::None
        },
        flat: args.is_present("flat"),
    }
}
