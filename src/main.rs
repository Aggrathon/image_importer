#[macro_use] extern crate lazy_static;
extern crate walkdir;
use std::fs::metadata;
use regex::Regex;
use chrono::{DateTime, Utc, TimeZone, Datelike};
use walkdir::{WalkDir, DirEntry};
use std::path::{Path, PathBuf};

fn main() {
    let meta = false;
    let name = false;
    let verbose = true;
    let source = "input";
    let dest = Path::new("output");
    let structure = OutputDirStructure::Swedish;

    WalkDir::new(source)
        .follow_links(true)
        .into_iter()
        .filter_entry(direntry_is_not_hidden)
        .filter_map(|v| v.ok()).for_each(|x: DirEntry| {
            if !x.file_type().is_file() { return; }
            let date = if name && !meta {
                get_date_from_name(&x.file_name().to_string_lossy())
            } else if name == meta{
                let tmp = get_date_from_name(&x.file_name().to_string_lossy());
                if tmp.is_ok() { tmp } else { get_date_from_meta(x.path()) }
            } else {
                get_date_from_meta(x.path())
            };
            match date {
                Ok(d) => {
                    let mut target = PathBuf::from(dest);
                    target.push(get_output_dir(&structure, d));
                    match std::fs::create_dir_all(&target) {
                        Result::Ok(_) => {
                            target.push(x.file_name());
                            if target.exists() {
                                println!("{}: Target already exists", x.path().display());
                            } else {
                                match std::fs::rename(x.path(), &target) {
                                    Result::Ok(_) => if verbose {println!("{}: Moved to {}", x.path().display(), target.display())},
                                    Result::Err(e) => println!("{}: {}", x.path().display(), e)
                                }
                            }
                        },
                        Result::Err(e) => println!("{}: {}", x.path().display(), e),
                    }
                },
                Err(e) => println!("{}: {}", x.path().display(), e)
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
    e.file_name().to_str().map(|s| e.depth() == 0 || !s.starts_with(".")).unwrap_or(false)
}
