use std::error::Error;
use std::cmp::Ordering;
use chrono::{NaiveDate, Datelike};
use serde::{Deserialize, Serialize};
use std::fmt;
use crate::date_serializer;
use std::env;
use std::env::VarError;

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    #[serde(with = "date_serializer")]
    date: NaiveDate,
    amount: f64,
    description: String,
    // switches: HashSet<String>,
    // tags: HashSet<String>,
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
        write!(f, "{}\t{:>7.2}\t{}", self.date, self.amount, self.description)
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
    for transaction in transactions {
        println!("{}", transaction);
    }
    Ok(())
}

fn write_entries(transactions: &mut Vec<Transaction>, filename: String) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;
    transactions.sort();
    for transaction in transactions {
        wtr.serialize(transaction)?;
    }
    wtr.flush()?;
    Ok(())
}

fn add_entry(year: u32, month: u32, day: u32, amount: f64, description: &str) -> Result<(), Box<dyn Error>> {
    let transaction = Transaction {
        date: NaiveDate::from_ymd(year as i32, month, day),
        amount,
        description: description.to_string()
    };
    let filename = get_filename_from_date(year, month)?;
    let mut transactions = get_transactions(&filename)?;
    transactions.push(transaction);
    write_entries(&mut transactions, filename)
}

pub fn add_date_entry(poss_date: &Option<String>, amount: f64, description: &str) -> Result<(), Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    add_entry(date.year() as u32, date.month(), date.day(), -amount, description)
}

pub fn print_date_list(poss_date: &Option<String>, is_detailed: bool) -> Result<(), Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    println!("------------------------------------------------------------");
    print_list(date.year() as u32, date.month())?;
    println!("------------------------------------------------------------");
    if is_detailed {
        print_sum_for_month(date.year() as u32, date.month())?;
    }
    Ok(())
}

fn get_date_or_today(poss_date: &Option<String>) -> Result<NaiveDate, chrono::ParseError> {
    match poss_date {
        None => { 
            let today = chrono::offset::Local::today();
            Ok(today.naive_local())
        },
        Some(date) => date_serializer::string_to_time(&date)
    }
}
