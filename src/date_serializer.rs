use chrono::NaiveDate;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

fn time_to_csv(t: NaiveDate) -> String {
    t.format("%Y-%m-%d").to_string()
}

pub fn string_to_time(s: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
}

pub fn serialize<S: Serializer>(time: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
    time_to_csv(*time).serialize(serializer)
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
    let time: String = Deserialize::deserialize(deserializer)?;
    string_to_time(&time).map_err(D::Error::custom)
}
