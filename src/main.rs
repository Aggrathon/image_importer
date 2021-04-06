#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate clap;
extern crate walkdir;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use clap::{App, Arg};
use regex::Regex;
use std::cmp::min;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn main() {
    let config = parse_args();
    move_and_sort(&config);
    if config.clean {
        clean_empty_dirs(config.input, config.verbose)
    }
}

enum Language {
    None,
    English,
    Swedish,
}

struct Config {
    input: PathBuf,
    output: PathBuf,
    verbose: bool,
    name: bool,
    meta: bool,
    clean: bool,
    min_year: i32,
    year: bool,
    month: Language,
    flat: bool,
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
                .help("Only get the dates from the filenames"),
        )
        .arg(
            Arg::with_name("meta")
                .long("meta")
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
                .short("y")
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
                .help("Flatten the directory structure (combine year and month)")
                .requires("year"),
        )
        .arg(
            Arg::with_name("INPUT")
                .takes_value(true)
                .required(true)
                .value_name("INPUT")
                .help("The input directory"),
        )
        .arg(
            Arg::with_name("OUPUT")
                .takes_value(true)
                .required(true)
                .value_name("OUPUT")
                .help("The ouput directory"),
        )
        .get_matches();
    let meta = args.is_present("meta");
    let name = args.is_present("name");
    Config {
        input: PathBuf::from(args.value_of("INPUT").unwrap()),
        output: PathBuf::from(args.value_of("OUTPUT").unwrap()),
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

fn move_and_sort(config: &Config) {
    WalkDir::new(&config.input)
        .follow_links(true)
        .min_depth(1)
        .into_iter()
        .filter_entry(direntry_is_not_hidden)
        .filter_map(|v| v.ok())
        .filter(|v| v.file_type().is_file())
        .for_each(|x: DirEntry| {
            let path = x.path();
            let filedate = if config.name {
                get_date_from_name(&x.file_name().to_string_lossy(), config.min_year)
            } else {
                Err(DateError::NotUsed)
            };
            let metadate = if config.meta {
                get_date_from_meta(&path)
            } else {
                Err(DateError::NotUsed)
            };
            let date = match filedate {
                Ok(fd) => match metadate {
                    Ok(md) => Ok(min(md, fd)),
                    Err(_) => Ok(fd),
                },
                Err(_) => metadate,
            };
            match date {
                Ok(d) => {
                    let mut target = config.output.join(get_output_dir(&config, d));
                    match std::fs::create_dir_all(&target) {
                        Result::Ok(_) => {
                            target.push(x.file_name());
                            if target == path {
                                if config.verbose {
                                    println!("{}: Already sorted", x.path().display());
                                }
                            } else if target.exists() {
                                println!("{}: Target already exists", x.path().display());
                            } else {
                                match std::fs::rename(&path, &target) {
                                    Result::Ok(_) => {
                                        if config.verbose {
                                            println!(
                                                "{}: Moved to {}",
                                                path.display(),
                                                target.display()
                                            )
                                        }
                                    }
                                    Result::Err(e) => println!("{}: {}", path.display(), e),
                                }
                            }
                        }
                        Result::Err(e) => println!("{}: {}", path.display(), e),
                    }
                }
                Err(e) => println!("{}: {}", path.display(), e),
            };
        });
}

#[derive(Debug)]
enum DateError {
    ParseError(std::num::ParseIntError),
    InvalidDate,
    InvalidDay,
    InvalidMonth,
    AncientDate,
    FutureDate,
    PatternMismatch,
    InvalidMetadata,
    IoError(std::io::Error),
    NotUsed,
}

impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DateError::ParseError(pie) => pie.fmt(f),
            DateError::InvalidDate => write!(f, "Invalid date"),
            DateError::InvalidDay => write!(f, "Invalid day"),
            DateError::InvalidMonth => write!(f, "Invalid month"),
            DateError::AncientDate => write!(f, "Year is too ancient"),
            DateError::FutureDate => write!(f, "Future date"),
            DateError::PatternMismatch => write!(f, "Date pattern not found"),
            DateError::InvalidMetadata => write!(f, "Metadata not found"),
            DateError::IoError(ioe) => ioe.fmt(f),
            &DateError::NotUsed => write!(f, "This date should not be used"),
        }
    }
}

impl std::error::Error for DateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DateError::ParseError(pie) => Some(pie),
            DateError::IoError(ioe) => Some(ioe),
            _ => None,
        }
    }
}

impl std::convert::From<std::io::Error> for DateError {
    fn from(error: std::io::Error) -> Self {
        DateError::IoError(error)
    }
}

impl std::convert::From<std::num::ParseIntError> for DateError {
    fn from(error: std::num::ParseIntError) -> Self {
        DateError::ParseError(error)
    }
}

fn get_date_from_meta(file: &Path) -> Result<DateTime<Utc>, DateError> {
    let meta = metadata(file)?;
    let date: Option<std::time::SystemTime> = meta
        .created()
        .into_iter()
        .chain(meta.modified().into_iter())
        .chain(meta.accessed().into_iter())
        .min();
    match date {
        Some(d) => Result::Ok(DateTime::from(d)),
        None => Err(DateError::InvalidMetadata),
    }
}

fn get_date_from_name(file: &str, min_year: i32) -> Result<DateTime<Utc>, DateError> {
    lazy_static! {
        static ref RGXS1: [Regex; 6] = [
            Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap(),
            Regex::new(r"(\d{4})_(\d{2})_(\d{2})").unwrap(),
            Regex::new(r"(\d{4})(\d{2})(\d{2})").unwrap(),
            Regex::new(r"(\d{4}) (\d{2}) (\d{2})").unwrap(),
            Regex::new(r"(\d{4}).(\d{2}).(\d{2})").unwrap(),
            Regex::new(r"(\d{4})/(\d{2})/(\d{2})").unwrap()
        ];
        static ref RGXS2: [Regex; 6] = [
            Regex::new(r"(\d{2})-(\d{2})-(\d{4})").unwrap(),
            Regex::new(r"(\d{2})_(\d{2})_(\d{4})").unwrap(),
            Regex::new(r"(\d{2})(\d{2})(\d{4})").unwrap(),
            Regex::new(r"(\d{2}) (\d{2}) (\d{4})").unwrap(),
            Regex::new(r"(\d{2}).(\d{2}).(\d{4})").unwrap(),
            Regex::new(r"(\d{2})/(\d{2})/(\d{4})").unwrap()
        ];
    }
    let mut date: Result<DateTime<Utc>, DateError> = Err(DateError::PatternMismatch);
    for rgx in RGXS1.iter() {
        for cap in rgx.captures_iter(file) {
            date = parse_time(&cap[1], &cap[2], &cap[3], min_year);
            if date.is_ok() {
                return date;
            }
        }
    }
    for rgx in RGXS2.iter() {
        for cap in rgx.captures_iter(file) {
            date = parse_time(&cap[3], &cap[2], &cap[1], min_year);
            if date.is_ok() {
                return date;
            }
        }
    }
    date
}

fn parse_time(
    year: &str,
    month: &str,
    day: &str,
    min_year: i32,
) -> Result<DateTime<Utc>, DateError> {
    lazy_static! {
        static ref NOW: DateTime<Utc> = Utc::now();
    }
    let year = year.parse::<i32>()?;
    let month = month.parse::<u32>()?;
    let day = day.parse::<u32>()?;
    if month == 0 || month > 12 {
        return Err(DateError::InvalidMonth);
    }
    if day == 0 || day > 31 {
        return Err(DateError::InvalidDay);
    }
    if year < min_year {
        return Err(DateError::AncientDate);
    }
    let date = Utc
        .ymd_opt(year, month, day)
        .single()
        .ok_or(DateError::InvalidDate)?
        .and_hms(0, 0, 1);
    if date > *NOW {
        Err(DateError::FutureDate)
    } else {
        Ok(date)
    }
}

fn get_output_dir(config: &Config, date: DateTime<Utc>) -> String {
    let year = date.year();
    let month = date.month();
    const MONTHS_EN: [&str; 12] = [
        " January",
        " February",
        " March",
        " April",
        " May",
        " June",
        " July",
        " August",
        " September",
        " October",
        " November",
        " December",
    ];
    const MONTHS_SWE: [&str; 12] = [
        " Januari",
        " Februari",
        " Mars",
        " April",
        " Maj",
        " Juni",
        " Juli",
        " Augusti",
        " September",
        " October",
        " November",
        " December",
    ];
    // SAFETY: months are 1-12
    let name = unsafe {
        match config.month {
            Language::None => "",
            Language::English => MONTHS_EN.get_unchecked((month - 1) as usize),
            Language::Swedish => MONTHS_SWE.get_unchecked((month - 1) as usize),
        }
    };
    if config.flat {
        format!("{:04} {:02}{}", year, month, name)
    } else if config.year {
        format!("{:04}/{:04} {:02}{}", year, year, month, name)
    } else {
        format!("{:04}/{:02}{}", year, month, name)
    }
}

fn clean_empty_dirs<P: AsRef<Path>>(path: P, verbose: bool) {
    WalkDir::new(path)
        .follow_links(true)
        .min_depth(1)
        .into_iter()
        .filter_entry(direntry_is_not_hidden)
        .filter_map(|v| v.ok())
        .filter(|v| v.file_type().is_dir())
        .for_each(|x: DirEntry| match std::fs::remove_dir(x.path()) {
            Result::Ok(_) => {
                if verbose {
                    println!("{}: Removed empty directory", x.path().display())
                }
            }
            Result::Err(_) => (),
        });
}

fn direntry_is_not_hidden(e: &DirEntry) -> bool {
    e.file_name()
        .to_str()
        .map(|s| !s.starts_with("."))
        .unwrap_or(false)
}
