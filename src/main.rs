#[macro_use] extern crate lazy_static;
#[macro_use] extern crate clap;
extern crate walkdir;
use std::fs::metadata;
use regex::Regex;
use chrono::{DateTime, Utc, TimeZone, Datelike};
use walkdir::{WalkDir, DirEntry};
use std::path::{Path, PathBuf};

fn main() {
    let (source, dest, meta, name, verbose, clean, structure) = parse_args();
    move_and_sort(&source, &dest, meta, name, verbose, structure);
    if clean {
        WalkDir::new(source)
        .follow_links(true)
        .min_depth(1)
        .into_iter()
        .filter_entry(direntry_is_not_hidden)
        .filter_map(|v| v.ok())
        .filter(|v| v.file_type().is_dir())
        .for_each(|x: DirEntry| {
            match std::fs::remove_dir(x.path()) {
                Result::Ok(_) => if verbose { println!("{}: Removed empty directory", x.path().display()) },
                Result::Err(_) => (),
            }
        });
    }
}

fn parse_args() -> (String, String, bool, bool, bool, bool, OutputDirStructure) {
    let matches = clap_app!(image_importer => 
        (version: "1.0")
        (author: "Aggrathon")
        (about: "Parses the filenames and metadata for all files in a directory (recursively) and moves them to another directory with a temporal hierarchy")
        (@arg verbose: -v --verbose "Prints messages for successful imports")
        (@arg name: -n --name "Only get the dates from the filenames")
        (@arg meta: -m --meta "Only get the dates from the metadata")
        (@arg clean: -c --clean "Remove empty directories from the input")
        (@arg STRUCTURE:default_value[Y_YM] possible_value[Y_YM Y_M YM Y_Mswe Y_Meng] -s --structure +takes_value "The temporal structure to use")
        (@arg INPUT: +required "Sets the input directory")
        (@arg OUTPUT: +required "Sets the output directory")
    ).get_matches();
    (
        matches.value_of("INPUT").unwrap().to_string(),
        matches.value_of("OUTPUT").unwrap().to_string(),
        matches.is_present("meta"),
        matches.is_present("name"),
        matches.is_present("verbose"),
        matches.is_present("clean"),
        match matches.value_of("STRUCTURE").unwrap() {
            "Y_M" => OutputDirStructure::Month,
            "YM" => OutputDirStructure::FlatYearMonth,
            "Y_YM" => OutputDirStructure::YearMonth,
            "Y_Mswe" => OutputDirStructure::Swedish,
            "Y_Meng" => OutputDirStructure::English,
            _ => OutputDirStructure::YearMonth
        }
    )
}

fn move_and_sort(source: &String, dest: &String, meta: bool, name: bool, verbose: bool, structure: OutputDirStructure) {
    WalkDir::new(source)
        .follow_links(true)
        .min_depth(1)
        .into_iter()
        .filter_entry(direntry_is_not_hidden)
        .filter_map(|v| v.ok())
        .filter(|v| v.file_type().is_file())
        .for_each(|x: DirEntry| {
            let path = x.path();
            let date = if name && !meta {
                get_date_from_name(&x.file_name().to_string_lossy())
            } else if name == meta{
                let tmp = get_date_from_name(&x.file_name().to_string_lossy());
                if tmp.is_ok() { tmp } else { get_date_from_meta(&path) }
            } else {
                get_date_from_meta(&path)
            };
            match date {
                Ok(d) => {
                    let mut target = PathBuf::from(&dest);
                    target.push(get_output_dir(&structure, d));
                    match std::fs::create_dir_all(&target) {
                        Result::Ok(_) => {
                            target.push(x.file_name());
                            if target == path {
                                if verbose { println!("{}: Already sorted", x.path().display()); }
                            } else if target.exists() {
                                println!("{}: Target already exists", x.path().display());
                            } else {
                                match std::fs::rename(&path, &target) {
                                    Result::Ok(_) => if verbose { println!("{}: Moved to {}", path.display(), target.display()) },
                                    Result::Err(e) => println!("{}: {}", path.display(), e)
                                }
                            }
                        },
                        Result::Err(e) => println!("{}: {}", path.display(), e),
                    }
                },
                Err(e) => println!("{}: {}", path.display(), e)
            };
    });
}

#[derive(Debug)]
enum DateError {
    ParseError(std::num::ParseIntError),
    InvalidDate,
    InvalidDay,
    InvalidMonth,
    FutureDate,
    PatternMismatch,
    IoError(std::io::Error)
}

impl std::fmt::Display for DateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DateError::ParseError(pie) => pie.fmt(f),
            DateError::InvalidDate => write!(f, "Invalid date"),
            DateError::InvalidDay => write!(f, "Invalid day"),
            DateError::InvalidMonth => write!(f, "Invalid month"),
            DateError::FutureDate => write!(f, "Future date"),
            DateError::PatternMismatch => write!(f, "Date pattern not found"),
            DateError::IoError(ioe) => ioe.fmt(f),
        }
    }
}

impl std::error::Error for DateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DateError::ParseError(pie) => Some(pie),
            DateError::IoError(ioe) => Some(ioe),
            _ => None
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
    let date = meta.created().or(meta.modified()).or(meta.accessed())?;
    let dt: DateTime<Utc> = DateTime::from(date);
    Result::Ok(dt)
}

fn get_date_from_name(file: &str) -> Result<DateTime<Utc>, DateError> {
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
            date = parse_time(&cap[1], &cap[2], &cap[3]);
            if date.is_ok() {
                return date;
            }
        }
    }
    for rgx in RGXS2.iter() {
        for cap in rgx.captures_iter(file) {
            date = parse_time(&cap[3], &cap[2], &cap[1]);
            if date.is_ok() {
                return date;
            }
        }
    }
    date
}

fn parse_time(year: &str, month: &str, day: &str) -> Result<DateTime<Utc>, DateError> {
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
    let date = Utc.ymd_opt(year, month, day).single().ok_or(DateError::InvalidDate)?.and_hms(0, 0, 1);
    if date > *NOW {
        Err(DateError::FutureDate)
    } else {
        Ok(date)
    }
}

enum OutputDirStructure {
    Month, YearMonth, FlatYearMonth, Swedish, English
}

fn get_output_dir(structure: &OutputDirStructure, date: DateTime<Utc>) -> String {
    let year = date.year();
    let month = date.month();
    match structure {
        OutputDirStructure::Month => format!("{}/{:02}", year, month),
        OutputDirStructure::YearMonth => format!("{y}/{y}-{m:02}", y=year, m=month),
        OutputDirStructure::FlatYearMonth => format!("{y}-{m:02}", y=year, m=month),
        OutputDirStructure::Swedish => format!("{}/{:02} {}", year, month, match month {
            1 => "Januari",
            2 => "Februari",
            3 => "Mars",
            4 => "April",
            5 => "Maj",
            6 => "Juni",
            7 => "Juli",
            8 => "Augusti",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Okänd"
        }),
        OutputDirStructure::English => format!("{}/{:02} {}", year, month, match month {
            1 => "January",
            2 => "February",
            3 => "March",
            4 => "April",
            5 => "May",
            6 => "June",
            7 => "July",
            8 => "August",
            9 => "September",
            10 => "October",
            11 => "November",
            12 => "December",
            _ => "Unknown"
        }),
    }
}

fn direntry_is_not_hidden(e: &DirEntry) -> bool {
    e.file_name().to_str().map(|s| !s.starts_with(".")).unwrap_or(false)
}
