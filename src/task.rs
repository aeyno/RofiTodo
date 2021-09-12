use chrono::NaiveDate;

/// A task struct
#[derive(Clone)]
pub struct Task {
    /// The content of the task
    pub name : String,
    /// An optionnal `NaiveDate` corresponding to when the task should be done
    pub date : Option<NaiveDate>
}

impl Task {
    /// Create a new `Task`
    /// 
    /// Create a new `Task` from its content
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the task
    pub fn new(name: String) -> Self {
        Task {name: name, date: None}
    }

    /// Create a new `Task`
    /// 
    /// Create a new `Task` from its content and date
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the task
    /// * `date` - the date when the task should be done
    pub fn new_with_date(name: String, date: NaiveDate) -> Self {
        Task {name: name, date: Some(date)}
    }

    /// Return a `String` representation of the task
    pub fn to_string(&self) -> String  {
        match self.date {
            Some(d) => format!("{} : {}", d.format("%Y-%m-%d"), self.name),
            None => format!("{}", self.name)
        }
    }

    /// Compare two `Task`s to sort them
    pub fn comp(&self, compare: &Self) -> std::cmp::Ordering {
        match (self.date, compare.date) {
            (Some(d1), Some(d2)) => if d1 == d2 {std::cmp::Ordering::Equal} else if d1 < d2 {std::cmp::Ordering::Less} else {std::cmp::Ordering::Greater},
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => if self.name.eq(&compare.name) {std::cmp::Ordering::Equal} else if self.name < compare.name {std::cmp::Ordering::Less} else {std::cmp::Ordering::Greater}
        }
    }
}

pub struct TaskList {
    content : Vec<Task>
}

impl TaskList {
    pub fn new() -> Self {
        TaskList { content : Vec::<Task>::new() }
    }

    pub fn _sort(&mut self) {
        self.content.sort_by(Task::comp);
    }

    fn binary_search(tab : &Vec<Task>, t: &Task) -> usize {
        let mut a : usize = 0;
        let mut b : usize = tab.len()-1;
        let mut m : usize;
        while a <= b {
            m  = (a+b)/2;
            match t.comp(&tab[m]) {
                std::cmp::Ordering::Greater => a = m+1,
                std::cmp::Ordering::Less => {
                    if m==0 { return a }
                    b = m-1;
                },
                std::cmp::Ordering::Equal => {
                    return m+1
                }, 
            }
        }
        return a;
    }

    pub fn get_content(&self) -> &Vec<Task> {
        &self.content
    }

    pub fn push(&mut self, t: Task) {
        if self.content.len() == 0 {
            self.content.push(t);
            return;
        }
        // We use binary search to improve the performances and sort the list will filling it
        self.content.insert(Self::binary_search(&self.content, &t), t);
    }

    pub fn remove(&mut self, index: usize) -> Task {
        self.content.remove(index)
    }
}



#[cfg(test)]
mod task_tests {
    use super::*;
    #[test]
    fn comp_date_nodate() {
        let t1 = Task::new(String::from("a task"));
        let t2 = Task::new_with_date(String::from("another task"), NaiveDate::from_ymd(2021, 1, 1));
        assert_eq!(t1.comp(&t2), std::cmp::Ordering::Less);
        assert_eq!(t2.comp(&t1), std::cmp::Ordering::Greater);
    }

    #[test]
    fn comp_nodate_nodate() {
        let t1 = Task::new(String::from("b"));
        let t2 = Task::new(String::from("c"));
        assert_eq!(t1.comp(&t2), std::cmp::Ordering::Less);
        let t3 = Task::new(String::from("a"));
        assert_eq!(t1.comp(&t3), std::cmp::Ordering::Greater);
    }

    #[test]
    fn comp_date_date() {
        let t1 = Task::new_with_date(String::from("a task"), NaiveDate::from_ymd(2021, 1, 1));
        let t2 = Task::new_with_date(String::from("another task"), NaiveDate::from_ymd(2021, 1, 2));
        assert_eq!(t1.comp(&t2), std::cmp::Ordering::Less);
        assert_eq!(t2.comp(&t1), std::cmp::Ordering::Greater);
    }
}

#[cfg(test)]
mod tasklist_tests {
    use super::*;

    #[test]
    fn push() {
        let mut tl = TaskList::new();
        let t1 = Task::new(String::from("a"));
        let t2 = Task::new(String::from("c"));
        let t3 = Task::new(String::from("b"));
        tl.push(t1.clone());
        tl.push(t2.clone());
        tl.push(t3.clone());
        assert_eq!(tl.get_content()[0].name, t1.name);
        assert_eq!(tl.get_content()[1].name, t3.name);
        assert_eq!(tl.get_content()[2].name, t2.name);
    }

    #[test]
    fn push_and_remove() {
        let mut tl = TaskList::new();
        let t1 = Task::new(String::from("a"));
        let t2 = Task::new(String::from("c"));
        let t3 = Task::new(String::from("b"));
        let t4 = Task::new(String::from("d"));
        tl.push(t1.clone());
        tl.push(t2.clone());
        tl.push(t3.clone());
        tl.remove(2);
        tl.push(t4.clone());
        assert_eq!(tl.get_content()[0].name, t1.name);
        assert_eq!(tl.get_content()[1].name, t3.name);
        assert_eq!(tl.get_content()[2].name, t4.name);
    }
}