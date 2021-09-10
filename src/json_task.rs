use crate::task::Task;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;


#[derive(Debug, Serialize, Deserialize)]
pub struct JsonTask {
    name : String,
    #[serde(skip_serializing_if = "Option::is_none")]
    date : Option<String>
}

impl JsonTask {
    pub fn from(t: &Task) -> Self {
        match t.date {
            Some(s) => {
                JsonTask { name :t.name.clone(), date: Some(s.format("%Y-%m-%d").to_string()) }
            },
            None => JsonTask { name :t.name.clone(), date: None }
        }
    }

    pub fn to_task(&self) -> Task{
        match &self.date {
            Some(s) => {
                Task::new_with_date(self.name.clone(), NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
            },
            None => Task::new(self.name.clone())
        }
    }
}