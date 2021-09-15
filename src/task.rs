use chrono::{NaiveDate, Local, Datelike};
use regex::{Regex, CaptureMatches, Captures};
use lazy_static::lazy_static;
use std::collections::HashMap;


#[derive(Clone)]
pub enum SortTaskBy {
    CreationDate,
    Content,
    Priority
}

/// A task struct
#[derive(Clone,Debug)]
pub struct Task {
    /// The content of the task
    pub content : String,
    /// An optionnal `NaiveDate` corresponding to when the task should be done
    duedate : Option<NaiveDate>,
    /// Is the task done
    pub completion : bool,
    /// When the task was completed
    pub completion_date : Option<NaiveDate>,
    /// When the task was created
    pub creation_date : Option<NaiveDate>,
    /// The priority, from A to Z
    pub priority : Option<char>,
    /// A list of project tags
    project_tags : Vec<String>,
    /// A list of context tags
    context_tags : Vec<String>,
    /// Custom tags with key and value
    custom_tags : HashMap<String,String>
}

impl Task {
    /// Create a new empty `Task`
    /// 
    /// Create a new `Task` from its content
    /// 
    /// Arguments:
    /// 
    /// * `content` - the content of the task
    pub fn empty() -> Self {
        Task {
            content: String::new(),
            duedate: None,
            completion : false,
            context_tags : vec![],
            project_tags : vec![],
            priority : None,
            creation_date : None,
            completion_date : None,
            custom_tags : HashMap::new()
        }
    }

    /// Create a new `Task`
    /// 
    /// Create a new `Task` from its content and add a creation date
    /// 
    /// Arguments:
    /// 
    /// * `content` - the content of the task
    pub fn new(content: String) -> Self {
        let today = Local::now();
        let mut task = Self::empty();
        task.set_content(content);
        task.creation_date = Some(NaiveDate::from_ymd(today.year(), today.month(), today.day()));
        task
    }

    /// Create a new `Task`
    /// 
    /// Create a new `Task` from its content and date
    /// 
    /// Arguments:
    /// 
    /// * `content` - the content of the task
    /// * `date` - the date when the task should be done
    pub fn new_with_date(content: String, date: NaiveDate) -> Self {
        let mut t = Self::new(content);
        t.set_due(Some(date));
        t
    }

    /// Change the content of a task
    /// 
    /// Change the content of the task and extract the new tags
    /// 
    /// Arguments:
    /// 
    /// * `content` - the new content of the task
    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.extract_tags();
    }

    pub fn get_content(&self) -> &String {
        &self.content
    }

    pub fn get_context_tags(&self) -> &Vec<String> {
        &self.context_tags
    }

    pub fn get_project_tags(&self) -> &Vec<String> {
        &self.project_tags
    }

    pub fn get_due(&self) -> &Option<NaiveDate> {
        &self.duedate
    }

    pub fn set_due(&mut self, date: Option<NaiveDate>) {
        self.duedate = date;
        match date {
            Some(date) => { self.custom_tags.insert(String::from("due"), format!("{}",date.format("%Y-%m-%d"))); },
            None => { self.custom_tags.remove_entry(&String::from("due")); }
        }
    }

    pub fn set_completed(&mut self) {
        self.completion = true;
        let today = Local::now();
        self.completion_date = Some(NaiveDate::from_ymd(today.year(), today.month(), today.day()));
        // Adding a creation date to respect the todo.txt specification (no task with a completion date and without a creation date)
        match self.creation_date {
            None => self.creation_date = Some(NaiveDate::from_ymd(today.year(), today.month(), today.day())),
            _ => ()
        }
    }

    pub fn set_not_completed(&mut self) {
        self.completion = false;
        self.completion_date = None;
    }

    /// Return a `String` representation of the task
    pub fn to_string(&self) -> String  {
        let mut s = String::new();
        if let Some(priority) = self.priority {
            s.push_str(&format!("({}) ", priority));
        }
        if let Some(date) = self.duedate {
            s.push_str(&format!("{} : ", date.format("%Y-%m-%d")));
        }
        s.push_str(&self.content);
        s
    }

    pub fn recap_str(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("𝐓𝐚𝐬𝐤 : {}", self.get_content()));
        if self.completion {
            s.push_str("\n𝐒𝐭𝐚𝐭𝐮𝐬 : Done");
            if let Some(date) = self.completion_date {
                s.push_str(&format!(" ({})",date.format("%Y-%m-%d")));
            }
        } else {
            s.push_str("\n𝐒𝐭𝐚𝐭𝐮𝐬 : To do");
        }
        if let Some(p) = self.priority {
            s.push_str(&format!("\n𝐏𝐫𝐢𝐨𝐫𝐢𝐭𝐲 : {}", p));
        }
        if let Some(date) = self.creation_date {
            s.push_str(&format!("\n𝐂𝐫𝐞𝐚𝐭𝐞𝐝 𝐨𝐧 : {}", date.format("%Y-%m-%d")));
        }
        if let Some(date) = self.duedate {
            s.push_str(&format!("\n𝐃𝐮𝐞 𝐝𝐚𝐭𝐞 : {}", date.format("%Y-%m-%d")));
        }
        if self.context_tags.len() > 0 {
            s.push_str(&format!{"\n𝐂𝐨𝐧𝐭𝐞𝐱𝐭 𝐭𝐚𝐠𝐬 : {}", self.get_context_tags().join(", ")});
        }
        if self.project_tags.len() > 0 {
            s.push_str(&format!{"\n𝐏𝐫𝐨𝐣𝐞𝐜𝐭 𝐭𝐚𝐠𝐬 : {}", self.get_project_tags().join(", ")});
        }
        s
    }

    pub fn from_todotxt(todo: String) -> Result<Self, String> {
        lazy_static! {
            static ref RE_TASK : Regex = Regex::new(r"^(?P<completion>x )?(\((?P<priority>[A-Z])\) )?(?P<compdate>\d{4}-\d{2}-\d{2} )?(?P<creadate>\d{4}-\d{2}-\d{2} )?(?P<content>.*)$").unwrap();
        }
        let cap : Captures;
        // Check if the String respects the todo.txt standard
        match RE_TASK.captures(&todo) {
            None => return Err(String::from("malformed task")),
            Some(result) => cap = result
        }


        let mut task = Self::new(String::new());
        match cap.name("completion") {
            Some(_) => task.completion = true,
            None => task.completion = false
        }
        match cap.name("priority") {
            Some(p) => task.priority = Some(p.as_str().chars().next().unwrap()),
            None => task.priority = None
        }

        // If there is only one date, it is a creation date
        // If there are two date, it is a completion date then a creation date
        match cap.name("creadate") {
            Some(creadate) => {
                task.creation_date = Some(NaiveDate::parse_from_str(&creadate.as_str(), "%Y-%m-%d ").unwrap());
                match cap.name("compdate") {
                    Some(compdate) => {
                        task.completion_date = Some(NaiveDate::parse_from_str(&compdate.as_str(), "%Y-%m-%d ").unwrap());
                    },
                    None => task.completion_date = None
                }
            }
            None => {
                match cap.name("compdate") {
                    Some(compdate) => {
                        task.creation_date = Some(NaiveDate::parse_from_str(&compdate.as_str(), "%Y-%m-%d ").unwrap());
                    },
                    None => {
                        task.completion_date = None;
                        task.creation_date = None;
                    }
                }
            }
        }

        // Extract content and custom tags
        let content = cap.name("content").unwrap().as_str();
        lazy_static! {
            static ref RE_ALLTAGS : Regex = Regex::new(r"( ([^:\s]+):([^:\s]+))+$").unwrap();
            static ref RE_TAG : Regex = Regex::new(r"(?P<key>[^:\s]+):(?P<value>[^:\s]+)").unwrap();
        }
        let alltags_result = RE_ALLTAGS.captures(&content);
        match alltags_result {
            None => task.content = String::from(content),
            Some(alltags) => {
                // Suppressing tags from the content
                task.content = String::from(&RE_ALLTAGS.replace_all(content, "").into_owned());
                // Iterate over all found tags
                for tag in RE_TAG.captures_iter(&alltags[0]) {
                    task.custom_tags.insert(String::from(tag.name("key").unwrap().as_str()), String::from(tag.name("value").unwrap().as_str()));
                }
            }
        }

        // Get Projet Tags and Context Tags
        task.extract_tags();

        // Extract the due date from custom tags
        match task.custom_tags.get(&String::from("due")) {
            Some(str_date) => task.duedate = match NaiveDate::parse_from_str(str_date.as_str(), "%Y-%m-%d ") {
                Ok(date) => Some(date),
                Err(_) => None
            },
            None => ()
        }
        Ok(task)
    }

    pub fn to_todotxt(&self) -> String {
        let mut s = String::new();
        if self.completion {
            s.push_str("x ");
        }
        if let Some(p) = self.priority {
            s.push_str(&format!("({}) ", p));
        }
        if let Some(date) = self.completion_date {
            s.push_str(&format!("{} ",date.format("%Y-%m-%d")));
        }
        if let Some(date) = self.creation_date {
            s.push_str(&format!("{} ",date.format("%Y-%m-%d")));
        }
        s.push_str(&self.content);
        for (key, value) in &self.custom_tags {
            s.push_str(&format!(" {}:{}", key, value));
        }
        s
    }

    /// Get project tags and context tags from task content
    fn extract_tags(&mut self) {
        lazy_static! {
            static ref RE_PROJECT_TAGS : Regex = Regex::new(r"((^| )\+(?P<tag>\S+))").unwrap();
            static ref RE_CONTEXT_TAGS : Regex = Regex::new(r"((^| )@(?P<tag>\S+))").unwrap();
        }
        self.project_tags = Self::get_tags_from_capture(RE_PROJECT_TAGS.captures_iter(&self.content));
        self.context_tags = Self::get_tags_from_capture(RE_CONTEXT_TAGS.captures_iter(&self.content));
    }

    /// Extract the tags from a Regex::CaptureMatches
    /// 
    /// Return a sorted and deduplicated `Vec<String>` with the tags
    fn get_tags_from_capture(captures : CaptureMatches) -> Vec<String> {
        let mut tags : Vec<String> = Vec::new();
        for tag in captures {
            tags.push(String::from(tag.name("tag").unwrap().as_str()));
        }
        tags.sort();
        tags.dedup();
        tags
    }

    /// Compare two `Task`s to sort them
    pub fn comp(&self, compare: &Self, sort: &SortTaskBy) -> std::cmp::Ordering {
        match sort {
            SortTaskBy::Content => {self.comp_content(compare)},
            SortTaskBy::CreationDate => {self.comp_creation_date(compare)},
            SortTaskBy::Priority =>{self.comp_priority(compare)}
        }
    }

    pub fn comp_priority(&self, compare: &Self) -> std::cmp::Ordering {
        match (self.priority, compare.priority) {
            (Some(p1), Some(p2)) => if p1 == p2 {self.comp_content(compare)} else if p1 < p2 {std::cmp::Ordering::Less} else {std::cmp::Ordering::Greater},
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => self.comp_content(compare)
        }
    }

    pub fn comp_creation_date(&self, compare: &Self) -> std::cmp::Ordering {
        match (self.creation_date, compare.creation_date) {
            (Some(d1), Some(d2)) => if d1 == d2 {self.comp_content(compare)} else if d1 < d2 {std::cmp::Ordering::Less} else {std::cmp::Ordering::Greater},
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => self.comp_content(compare)
        }
    }

    pub fn comp_content(&self, compare: &Self) -> std::cmp::Ordering {
        if self.content == compare.content {
            std::cmp::Ordering::Equal
        } else if self.content < compare.content {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

pub struct TaskList {
    content : Vec<Task>,
    sort : SortTaskBy
}

impl TaskList {
    pub fn new() -> Self {
        TaskList { content : Vec::<Task>::new(), sort : SortTaskBy::CreationDate }
    }

    pub fn change_sort(&mut self, new_sort: SortTaskBy) {
        self.sort = new_sort;
        self.sort();
    }

    pub fn sort(&mut self) {
        match self.sort {
            SortTaskBy::Content => {self.content.sort_by(Task::comp_content)},
            SortTaskBy::CreationDate => {self.content.sort_by(Task::comp_creation_date)},
            SortTaskBy::Priority =>{self.content.sort_by(Task::comp_priority)}
        }
    }

    fn binary_search(tab : &Vec<Task>, t: &Task, sort : &SortTaskBy) -> usize {
        let mut a : usize = 0;
        let mut b : usize = tab.len()-1;
        let mut m : usize;
        while a <= b {
            m  = (a+b)/2;
            match t.comp(&tab[m], &sort) {
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
        // We use binary search to improve the performances and sort the list while filling it
        self.content.insert(Self::binary_search(&self.content, &t, &self.sort), t);
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
        let t1 = Task::from_todotxt(String::from("a task")).unwrap();
        let t2 = Task::from_todotxt(String::from("2021-01-01 another task")).unwrap();
        assert_eq!(t1.comp_creation_date(&t2), std::cmp::Ordering::Less);
        assert_eq!(t2.comp_creation_date(&t1), std::cmp::Ordering::Greater);
    }

    #[test]
    fn comp_nodate_nodate() {
        let t1 = Task::new(String::from("b"));
        let t2 = Task::new(String::from("c"));
        assert_eq!(t1.comp_content(&t2), std::cmp::Ordering::Less);
        let t3 = Task::new(String::from("a"));
        assert_eq!(t1.comp_content(&t3), std::cmp::Ordering::Greater);
    }

    #[test]
    fn comp_date_date() {
        let t1 = Task::from_todotxt(String::from("a task")).unwrap();
        let t2 = Task::from_todotxt(String::from("another task")).unwrap();
        assert_eq!(t1.comp_creation_date(&t2), std::cmp::Ordering::Less);
        assert_eq!(t2.comp_creation_date(&t1), std::cmp::Ordering::Greater);
    }

    #[test]
    fn from_todotxt() {
        let t1 = Task::from_todotxt(String::from("(A) Thank Mom for the aaa @phone")).unwrap();
        assert_eq!(t1.get_content(), "Thank Mom for the aaa @phone");
        assert_eq!(t1.creation_date, None);
        assert_eq!(t1.completion_date, None);
        assert_eq!(t1.completion, false);
        assert_eq!(t1.priority, Some('A'));
        assert_eq!(*t1.get_context_tags(), vec!["phone"]);
        assert_eq!(*t1.get_project_tags(), Vec::<String>::new());

        let t2 = Task::from_todotxt(String::from("(B) Schedule Goodwill pickup +GarageSale @phone")).unwrap();
        assert_eq!(t2.get_content(), "Schedule Goodwill pickup +GarageSale @phone");
        assert_eq!(t2.creation_date, None);
        assert_eq!(t2.completion_date, None);
        assert_eq!(t2.completion, false);
        assert_eq!(t2.priority, Some('B'));
        assert_eq!(*t2.get_context_tags(), vec!["phone"]);
        assert_eq!(*t2.get_project_tags(), vec!["GarageSale"]);

        let t3 = Task::from_todotxt(String::from("x Post signs around the neighborhood +GarageSale")).unwrap();
        assert_eq!(t3.get_content(), "Post signs around the neighborhood +GarageSale");
        assert_eq!(t3.creation_date, None);
        assert_eq!(t3.completion_date, None);
        assert_eq!(t3.completion, true);
        assert_eq!(t3.priority, None);
        assert_eq!(*t3.get_context_tags(), Vec::<String>::new());
        assert_eq!(*t3.get_project_tags(), vec!["GarageSale"]);

        let t4 = Task::from_todotxt(String::from("2021-09-01 @GroceryStore Eskimo pies")).unwrap();
        assert_eq!(t4.get_content(), "@GroceryStore Eskimo pies");
        assert_eq!(format!("{}", t4.creation_date.unwrap().format("%Y-%m-%d")), "2021-09-01");
        assert_eq!(t4.completion_date, None);
        assert_eq!(t4.completion, false);
        assert_eq!(t4.priority, None);
        assert_eq!(*t4.get_context_tags(), vec!["GroceryStore"]);
        assert_eq!(*t4.get_project_tags(), Vec::<String>::new());
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
        assert_eq!(tl.get_content()[0].content, t1.content);
        assert_eq!(tl.get_content()[1].content, t3.content);
        assert_eq!(tl.get_content()[2].content, t2.content);
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
        assert_eq!(tl.get_content()[0].content, t1.content);
        assert_eq!(tl.get_content()[1].content, t3.content);
        assert_eq!(tl.get_content()[2].content, t4.content);
    }
}