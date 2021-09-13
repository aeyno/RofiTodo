mod rofi;
use rofi::{Rofi, RofiParams};
mod task;
use task::{Task, TaskList};
mod json_task;
use json_task::JsonTask;
mod date_selector;
use date_selector::date_selector;
use serde_json;
use std::collections::HashMap;
use std::fs;
use structopt::StructOpt;
use chrono::{Utc,NaiveDate};
use chrono::Datelike;


#[derive(StructOpt)]
struct Cli {
    /// The path to the RofiTodo config/task list file
    #[structopt(short, long, parse(from_os_str), default_value = "./rofitodo")]
    config: std::path::PathBuf,
    /// Do not load Rofi configuration, use default values.
    #[structopt(long = "no-config")]
    no_config: bool,
    /// Set filter to be case insensitive
    #[structopt(short = "i", long = "case-insensitive")]
    case_insensitive: bool
}

fn show_task_menu(rofi_config : &RofiParams, todos : &mut TaskList, oldlist: &mut TaskList, index: usize) -> bool {
    let mut menu =  vec![String::from("✔ mark as done"), String::from("+ edit"), String::from("+ change date")];
    match todos.get_content().get(index).unwrap().date {
        Some(_) => menu.push(String::from("! remove date")),
        None => ()
    }
    menu.push(String::from("* cancel"));
    match Rofi::from(rofi_config).prompt("Edit").run(menu).unwrap().as_ref() {
        "✔ mark as done" => {
            oldlist.push(todos.remove(index));
            true
        },
        "* cancel" => true,
        "+ edit" => {
            let task = Rofi::from(rofi_config).prompt("Task").placeholder("").text_only().run(vec![]).unwrap();
            if task.eq("") {
                return false;
            }
            let old_task = todos.remove(index);
            match old_task.date {
                Some(date) => {
                    todos.push(Task::new_with_date(task, date))
                },
                None => todos.push(Task::new(task))
            }
            true
        },
        "+ change date" => {
            let now = Utc::now();
            match date_selector(rofi_config, NaiveDate::from_ymd(now.year(), now.month(), now.day())) {
                Some(date) => {
                    let old_task = todos.remove(index);
                    todos.push(Task::new_with_date(old_task.name, date))
                },
                None => ()
            }
            true
        },
        "! remove date" => {
            let old_task = todos.remove(index);
            todos.push(Task::new(old_task.name));            
            true
        },
        _ => false
    }
}

fn show_add_task(rofi_config : &RofiParams, todos : &mut TaskList) -> bool {
    let task = Rofi::from(rofi_config).prompt("Task").placeholder("").text_only().run(vec![]).unwrap();
    if task.eq("") {
        return false;
    }
    let menu =  vec![String::from("✔ validate"), String::from("+ add date"), String::from("* cancel")];
    match Rofi::from(rofi_config).prompt("Edit").run(menu).unwrap().as_ref() {
        "✔ validate" => {
            todos.push(Task::new(task));
            true
        },
        "* cancel" => true,
        "+ add date" => {
            let now = Utc::now();
            match date_selector(rofi_config, NaiveDate::from_ymd(now.year(), now.month(), now.day())) {
                Some(date) => todos.push(Task::new_with_date(task, date)),
                None => ()
            }
            true
        },
        _ => false
    }
}

fn show_old_menu(rofi_config : &RofiParams, oldlist: &mut TaskList) -> bool {
    let mut choices =  vec![String::from("← back"), String::from("@ exit")];
    for todo in oldlist.get_content() {
        choices.push(todo.to_string());
    }
    match Rofi::from(rofi_config).prompt("Todo").run(choices).unwrap().as_ref() {
        "← back" => true,
        "@ exit" => false,
        "" => false,
        s => {
            let index = oldlist.get_content().iter().position(|x| x.to_string().eq(s));
            match index {
                Some(i) => {
                    oldlist.remove(i);
                },
                None => ()
            }
            true
        }
    }
}

fn show_main_menu(rofi_config : &RofiParams, todos : &mut TaskList, oldlist: &mut TaskList) -> bool {
    let mut choices = vec![String::from("+ add"), String::from("~ old"), String::from("@ exit")];
    for todo in todos.get_content() {
        choices.push(todo.to_string());
    }
    match Rofi::from(rofi_config).prompt("Todo").run(choices).unwrap().as_ref() {
        "+ add" => {
            show_add_task(rofi_config, todos);
            true
        },
        "~ old" => {
            show_old_menu(rofi_config, oldlist)
        },
        "@ exit" => false,
        "" => false,
        s => {
            let index = todos.get_content().iter().position(|x| x.to_string().eq(s));
            match index {
                Some(i) => {show_task_menu(rofi_config, todos, oldlist, i);},
                None => ()
            }
            true
        }
    }
}

fn load_config(config_file: &std::path::PathBuf, todos: &mut TaskList, old: &mut TaskList) -> Result<bool, String> {
    if !std::path::Path::new(config_file).exists() {
        save_config(config_file, &mut TaskList::new(), &mut TaskList::new()).unwrap();
    }
    let read = fs::read_to_string(config_file);
    match read {
        Ok(_) => (),
        Err(e) => return Err(e.to_string())
    }
    let content = read.unwrap();
    let parse_result = serde_json::from_str(&content);
    match parse_result {
        Ok(_) => (),
        Err(e) => return Err(format!("{} : {}", "Bad json", e.to_string()))
    }
    let parsed : HashMap<String,Vec<JsonTask>> = parse_result.unwrap();

    for i in 0..parsed["todos"].len() {
        todos.push(parsed["todos"][i].to_task());
    }
    for i in 0..parsed["old"].len() {
        old.push(parsed["old"][i].to_task());
    }
    Ok(true)
}

fn save_config(config_file: &std::path::PathBuf, todos: &mut TaskList, old: &mut TaskList) -> Result<bool,String> {
    let mut save = HashMap::<String, Vec<JsonTask>>::new();
    save.entry(String::from("todos")).or_insert(Vec::<JsonTask>::new());
    for todo in todos.get_content() {
        save.entry(String::from("todos")).or_insert(Vec::<JsonTask>::new()).push(JsonTask::from(&todo));
    }
    save.entry(String::from("old")).or_insert(Vec::<JsonTask>::new());
    for todo in old.get_content() {
        save.entry(String::from("old")).or_insert(Vec::<JsonTask>::new()).push(JsonTask::from(&todo));
    }
    let res = serde_json::to_string(&save);

    match fs::write(config_file, res.unwrap()) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string())
    }
}

fn main() {
    let mut todos = TaskList::new();
    let mut old = TaskList::new();

    let args = Cli::from_args();

    let rofi_config = RofiParams { no_config : args.no_config, case_insensitive : args.case_insensitive };

    let config = args.config;
    match load_config(&config, &mut todos, &mut old) {
        Ok(_) => (),
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    loop {
        if !show_main_menu(&rofi_config, &mut todos, &mut old) { break }
    }

    match save_config(&config, &mut todos, &mut old) {
        Ok(_) => (),
        Err(s) => println!("{}", s)
    };
}
