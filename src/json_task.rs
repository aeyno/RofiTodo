use crate::task::Task;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;


/// A task struct which can be serialised to JSON using `Serde`
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonTask {
    /// The content of the task
    name : String,
    /// An optional date string with the YYYY-MM-DD format
    #[serde(skip_serializing_if = "Option::is_none")]
    date : Option<String>
}

impl JsonTask {
    /// Transforms a `Task` struct to a `JsonTask` struct which can then be serialised to JSON
    /// 
    /// Returns a `JsonTask`
    /// 
    /// Arguments:
    /// 
    /// * `t` - the `Task` to convert to `JsonTask`
    pub fn from(t: &Task) -> Self {
        match t.date {
            Some(s) => {
                JsonTask { name :t.name.clone(), date: Some(s.format("%Y-%m-%d").to_string()) }
            },
            None => JsonTask { name :t.name.clone(), date: None }
        }
    }

    /// Transforms a `JsonTask` struct to a `Task` struct
    /// 
    /// Returns a `Task`
    pub fn to_task(&self) -> Task{
        match &self.date {
            Some(s) => {
                Task::new_with_date(self.name.clone(), NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
            },
            None => Task::new(self.name.clone())
        }
    }
}

#[cfg(test)]
mod jsontask_tests {
    use super::*;
    #[test]
    fn from_to() {
        let t1 = Task::new(String::from("a task"));
        let t2 = JsonTask::from(&t1).to_task();
        assert_eq!(t2.name, t1.name);
        assert_eq!(t2.date, t1.date);
        let t3 = Task::new_with_date(String::from("another task"), NaiveDate::from_ymd(2021, 1, 1));
        let t4 = JsonTask::from(&t3).to_task();
        assert_eq!(t4.name, t3.name);
        assert_eq!(t4.date, t3.date);
    }

    #[test]
    fn to_from() {
        let t1 = JsonTask {name : String::from("test json task"), date : None};
        let t2 = t1.to_task();
        assert_eq!(t2.name, t1.name);
        assert_eq!(t2.date, None);
        let t3 =  JsonTask {name : String::from("test json task"), date : Some(String::from("2021-01-01"))};
        let t4 = t3.to_task();
        assert_eq!(t4.name, t3.name);
        assert_eq!(t4.date, Some(NaiveDate::from_ymd(2021, 1, 1)));
    }
}