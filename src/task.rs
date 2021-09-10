use chrono::NaiveDate;

#[derive(Clone)]
pub struct Task {
    pub name : String,
    pub date : Option<NaiveDate>
}

impl Task {
    pub fn new(name: String) -> Self {
        Task {name: name, date: None}
    }

    pub fn new_with_date(name: String, date: NaiveDate) -> Self {
        Task {name: name, date: Some(date)}
    }

    pub fn to_string(&self) -> String  {
        match self.date {
            Some(d) => format!("{} : {}", d.format("%Y-%m-%d"), self.name),
            None => format!("{}", self.name)
        }
    }

    pub fn _comp(&self, compare: &Self) -> std::cmp::Ordering {
        match (self.date, compare.date) {
            (Some(d1), Some(d2)) => if d1 == d2 {std::cmp::Ordering::Equal} else if d1 < d2 {std::cmp::Ordering::Less} else {std::cmp::Ordering::Greater},
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal
        }
    }
}

pub struct TaskList {
    pub content : Vec<Task>
}

impl TaskList {
    pub fn new() -> Self {
        TaskList { content : Vec::<Task>::new() }
    }

    pub fn sort(&mut self) {
        self.content.sort_by(Task::_comp);
    }

    pub fn push(&mut self, t: Task) {
        self.content.push(t);
        self.sort();
    }
}