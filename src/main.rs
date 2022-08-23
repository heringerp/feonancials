use std::error::Error;
use std::io;
use std::process;
use chrono::NaiveDate;

const PATH: &str = "/home/heringer/Documents/ImportantDocs/financials";

use serde::Deserialize;

mod date_serializer {
    use chrono::NaiveDate;
    use serde::{Serializer, Deserializer, Serialize, Deserialize, de::Error};

    fn time_to_csv(t: NaiveDate) -> String {
        t.format("%Y-%m-%d").to_string()
    }

    pub fn serialize<S: Serializer>(time: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
        time_to_csv(time.clone()).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
        let time: String = Deserialize::deserialize(deserializer)?;
        Ok(NaiveDate::parse_from_str(&time, "%Y-%m-%d").map_err(D::Error::custom)?)
    }

}

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(with = "date_serializer")]
    date: NaiveDate,
    amount: f64,
    description: String,
}

fn get_filename_from_date(year: u32, month: u32) -> String {
    format!("{}/{}/{:0>2}.csv", PATH, year, month)
}

fn get_transactions(filename: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut transactions = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_path(filename)?;
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: Transaction = result?;
        println!("{:?}", record);
        transactions.push(record);
    }
    Ok(transactions)
}

fn get_sum_for_month(year: u32, month: u32) -> Result<f64, Box<dyn Error>> {
    let filename = get_filename_from_date(year, month);
    let transactions = get_transactions(&filename)?;
    Ok(transactions.into_iter().map(|x| x.amount).sum())
}

fn main() {
    println!("{}", get_filename_from_date(2022, 6));
    match get_sum_for_month(2022, 6) {
        Ok(r) => println!("Sum: {}", r),
        Err(r) => println!("Err: {}", r)
    }
}
