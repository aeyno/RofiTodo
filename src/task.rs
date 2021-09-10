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