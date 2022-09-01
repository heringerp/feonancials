use crate::date_serializer;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::env;
use std::env::VarError;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(with = "date_serializer")]
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
    // switches: HashSet<String>,
    // tags: HashSet<String>,
}

impl Default for Transaction {
    fn default() -> Transaction {
        Transaction {
            date: chrono::offset::Local::today().naive_local(),
            amount: 0.0,
            description: String::new(),
        }
    }
}

impl Eq for Transaction {}

impl Ord for Transaction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for Transaction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date && self.description == other.description
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{:>7.2}\t{}",
            self.date, self.amount, self.description
        )
    }
}

fn get_base_path() -> Result<String, VarError> {
    env::var("FEONANCIALS_PATH")
}

fn get_filename_from_date(year: u32, month: u32) -> Result<String, VarError> {
    let base_path = get_base_path();
    Ok(format!("{}/{}/{:0>2}.csv", base_path?, year, month))
}

fn get_transactions(filename: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut transactions = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        // .has_headers(false)
        .trim(csv::Trim::All)
        .from_path(filename)?;
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: Transaction = result?;
        // println!("{:?}", record);
        transactions.push(record);
    }
    Ok(transactions)
}

fn get_sum_for_month(year: u32, month: u32) -> Result<f64, Box<dyn Error>> {
    let filename = get_filename_from_date(year, month)?;
    let transactions = get_transactions(&filename)?;
    Ok(transactions.into_iter().map(|x| x.amount).sum())
}

fn print_sum_for_month(year: u32, month: u32) -> Result<(), Box<dyn Error>> {
    let sum = get_sum_for_month(year, month)?;
    println!("Sum:\t\t{:>7.2}", sum);
    Ok(())
}

fn print_list(year: u32, month: u32) -> Result<(), Box<dyn Error>> {
    let filename = get_filename_from_date(year, month)?;
    let transactions = get_transactions(&filename)?;
    for (index, transaction) in transactions.iter().enumerate() {
        println!("{:>3}  {}", index, transaction);
    }
    Ok(())
}

fn write_entries(
    transactions: &mut Vec<Transaction>,
    filename: String,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;
    transactions.sort();
    for transaction in transactions {
        wtr.serialize(transaction)?;
    }
    wtr.flush()?;
    Ok(())
}

fn add_entry(
    year: u32,
    month: u32,
    day: u32,
    amount: f64,
    description: &str,
) -> Result<(), Box<dyn Error>> {
    let transaction = Transaction {
        date: NaiveDate::from_ymd(year as i32, month, day),
        amount,
        description: description.to_string(),
    };
    let filename = get_filename_from_date(year, month)?;
    let mut transactions = get_transactions(&filename)?;
    transactions.push(transaction);
    write_entries(&mut transactions, filename)
}

pub fn add_transaction() {
    // TODO
}

pub fn add_date_entry(
    poss_date: &Option<String>,
    amount: f64,
    description: &str,
) -> Result<(), Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    add_entry(
        date.year() as u32,
        date.month(),
        date.day(),
        -amount,
        description,
    )
}

pub fn print_date_list(
    poss_date: &Option<String>,
    is_detailed: bool,
) -> Result<(), Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    println!("------------------------------------------------------------");
    print_list(date.year() as u32, date.month())?;
    println!("------------------------------------------------------------");
    if is_detailed {
        print_sum_for_month(date.year() as u32, date.month())?;
    }
    Ok(())
}

pub fn get_transactions_for_month(
    poss_date: &Option<String>,
) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    let filename = get_filename_from_date(date.year() as u32, date.month())?;
    let mut transactions = get_transactions(&filename)?;
    transactions.sort();
    Ok(transactions)
}

pub fn del_entry(poss_date: &Option<String>, index: usize) -> Result<(), Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    let filename = get_filename_from_date(date.year() as u32, date.month())?;
    let mut transactions = get_transactions(&filename)?;
    transactions.remove(index);
    write_entries(&mut transactions, filename)
}

pub fn get_date_or_today(poss_date: &Option<String>) -> Result<NaiveDate, chrono::ParseError> {
    match poss_date {
        None => {
            let today = chrono::offset::Local::today();
            Ok(today.naive_local())
        }
        Some(date) => date_serializer::string_to_time(date),
    }
}

pub fn get_months() -> Result<Vec<String>, Box<dyn Error>> {
    let base_path_string = get_base_path()?;
    let base_path = Path::new(&base_path_string);
    let mut result = Vec::new();
    for entry in fs::read_dir(base_path)? {
        let entry = entry?;
        let path = entry.path();
        let path_copy = path.clone();
        let path_stem = path_copy.file_stem().unwrap().to_str().unwrap();
        if !path.is_dir() {
            continue;
        }
        for month in fs::read_dir(path)? {
            let month = month?;
            let month_path = month.path();
            if month_path.is_dir() {
                continue;
            }
            let text = format!(
                "{}-{}",
                path_stem,
                month_path.file_stem().unwrap().to_str().unwrap()
            );
            result.push(text);
        }
    }
    result.sort();
    Ok(result)
}
