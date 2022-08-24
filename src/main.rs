use std::error::Error;
use chrono::{NaiveDate, Datelike};
use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};
use std::fmt;

const PATH: &str = "/home/heringer/Documents/ImportantDocs/financials";

mod date_serializer {
    use chrono::NaiveDate;
    use serde::{Serializer, Deserializer, Serialize, Deserialize, de::Error};

    fn time_to_csv(t: NaiveDate) -> String {
        t.format("%Y-%m-%d").to_string()
    }

    pub fn string_to_time(s: &str) -> Result<NaiveDate, chrono::ParseError> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
    }

    pub fn serialize<S: Serializer>(time: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
        time_to_csv(time.clone()).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(string_to_time(&time).map_err(D::Error::custom)?)
    }

}

#[derive(Parser)]
struct Arguments {
    #[clap(subcommand)]
    command: Option<Commands>

}

#[derive(Subcommand)]
enum Commands {
    Add {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(value_parser)]
        amount: f64,

        #[clap(value_parser)]
        description: String
    },

    List {
        #[clap(long, short, action)]
        date: Option<String>,

        #[clap(long, short, action)]
        full: bool,
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    #[serde(with = "date_serializer")]
    date: NaiveDate,
    amount: f64,
    description: String,
    // switches: HashSet<String>,
    // tags: HashSet<String>,
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{:>7.2}\t{}", self.date, self.amount, self.description)
    }
}

fn get_filename_from_date(year: u32, month: u32) -> String {
    format!("{}/{}/{:0>2}.csv", PATH, year, month)
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
    let filename = get_filename_from_date(year, month);
    let transactions = get_transactions(&filename)?;
    Ok(transactions.into_iter().map(|x| x.amount).sum())
}

fn print_sum_for_month(year: u32, month: u32) -> Result<(), Box<dyn Error>> {
    let sum = get_sum_for_month(year, month)?;
    println!("Sum:\t\t{:>7.2}", sum);
    Ok(())
}

fn print_list(year: u32, month: u32) -> Result<(), Box<dyn Error>> {
    let filename = get_filename_from_date(year, month);
    let transactions = get_transactions(&filename)?;
    for transaction in transactions {
        println!("{}", transaction);
    }
    Ok(())
}

fn write_entries(transactions: Vec<Transaction>, filename: String) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;
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
    let filename = get_filename_from_date(year, month);
    let mut transactions = get_transactions(&filename)?;
    transactions.push(transaction);
    write_entries(transactions, filename)
}

fn add_date_entry(poss_date: &Option<String>, amount: f64, description: &str) -> Result<(), Box<dyn Error>> {
    let date = get_date_or_today(poss_date)?;
    add_entry(date.year() as u32, date.month(), date.day(), -amount, description)
}

fn print_date_list(poss_date: &Option<String>, is_detailed: bool) -> Result<(), Box<dyn Error>> {
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

fn main() {
    println!("{}", get_filename_from_date(2022, 9));
    let arg = Arguments::parse();
    let command = &arg.command;
    if command.is_none() {
    } else {
        let res = match &arg.command.unwrap() {
            Commands::Add { date, amount, description } => add_date_entry(date, *amount, description),
            Commands::List { date, full } => print_date_list(date, *full),
        };
        if let Err(r) = res {
            eprintln!("{}", r);
            std::process::exit(1)
        }
    }
}
