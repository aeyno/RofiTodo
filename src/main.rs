mod rofi;
use rofi::Rofi;
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


#[derive(StructOpt)]
struct Cli {
    /// The path to the config file
    #[structopt(short, long, parse(from_os_str), default_value = "./rofitodo")]
    config: std::path::PathBuf,
}

fn show_task_menu(todos : &mut TaskList, oldlist: &mut TaskList, index: usize) -> bool {
    let mut menu =  vec![String::from("✔ mark as done"), String::from("+ edit"), String::from("+ change date")];
    match todos.get_content().get(index).unwrap().date {
        Some(_) => menu.push(String::from("! remove date")),
        None => ()
    }
    menu.push(String::from("* cancel"));
    match Rofi::new().prompt("Edit").run(menu).unwrap().as_ref() {
        "✔ mark as done" => {
            oldlist.push(todos.remove(index));
            true
        },
        "* cancel" => true,
        "+ edit" => {
            let task = Rofi::new().prompt("Task").placeholder("").text_only().run(vec![]).unwrap();
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
            match date_selector() {
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

fn show_add_task(todos : &mut TaskList) -> bool {
    let task = Rofi::new().prompt("Task").placeholder("").text_only().run(vec![]).unwrap();
    if task.eq("") {
        return false;
    }
    let menu =  vec![String::from("✔ validate"), String::from("+ add date"), String::from("* cancel")];
    match Rofi::new().prompt("Edit").run(menu).unwrap().as_ref() {
        "✔ validate" => {
            todos.push(Task::new(task));
            true
        },
        "* cancel" => true,
        "+ add date" => {
            match date_selector() {
                Some(date) => todos.push(Task::new_with_date(task, date)),
                None => ()
            }
            true
        },
        _ => false
    }
}

fn show_old_menu(oldlist: &mut TaskList) -> bool {
    let mut choices =  vec![String::from("← back"), String::from("@ exit")];
    for todo in oldlist.get_content() {
        choices.push(todo.to_string());
    }
    match Rofi::new().prompt("Todo").run(choices).unwrap().as_ref() {
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

fn show_main_menu(todos : &mut TaskList, oldlist: &mut TaskList) -> bool {
    let mut choices = vec![String::from("+ add"), String::from("~ old"), String::from("@ exit")];
    for todo in todos.get_content() {
        choices.push(todo.to_string());
    }
    match Rofi::new().prompt("Todo").run(choices).unwrap().as_ref() {
        "+ add" => {
            show_add_task(todos);
            true
        },
        "~ old" => {
            show_old_menu(oldlist)
        },
        "@ exit" => false,
        "" => false,
        s => {
            let index = todos.get_content().iter().position(|x| x.to_string().eq(s));
            match index {
                Some(i) => {show_task_menu(todos, oldlist, i);},
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
    let parsed : HashMap<String,Vec<JsonTask>> = serde_json::from_str(&content).expect("Invalid config file");

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

    let config = args.config;
    match load_config(&config, &mut todos, &mut old) {
        Ok(_) => (),
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    loop {
        if !show_main_menu(&mut todos, &mut old) { break }
    }

    match save_config(&config, &mut todos, &mut old) {
        Ok(_) => (),
        Err(s) => println!("{}", s)
    };
}
