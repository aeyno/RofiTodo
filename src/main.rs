mod rofi;
use rofi::{Rofi, RofiParams};
mod task;
use task::{Task, TaskList, SortTaskBy};
mod date_selector;
use date_selector::date_selector;
use std::fs;
use std::io::{self, BufRead};
use structopt::StructOpt;
use chrono::{Local, NaiveDate, Datelike};

#[derive(StructOpt)]
struct Cli {
    /// The path to the RofiTodo config/task list file
    #[structopt(short, long, parse(from_os_str), default_value = "./todo.txt")]
    config: std::path::PathBuf,
    /// Do not load Rofi configuration, use default values.
    #[structopt(long = "no-config")]
    no_config: bool,
    /// Set filter to be case insensitive
    #[structopt(short = "i", long = "case-insensitive")]
    case_insensitive: bool,
    /// How to sort the tasks
    #[structopt(short = "s", long="sort", possible_values = &["creation","content","priority"], case_insensitive = true, default_value="content")]
    sort : String
}

#[derive(PartialEq)]
enum MenuStatus {
    EXIT,
    MAINMENU,
    BACK
}

fn show_task_menu(rofi_config : &RofiParams, todos : &mut TaskList, done: &mut TaskList, index: usize) -> MenuStatus {
    loop {
        let mut menu =  vec![String::from("✔ mark as done"), String::from("+ edit"), String::from("+ change date")];
        match todos.get_content().get(index).unwrap().get_due() {
            Some(_) => menu.push(String::from("! remove date")),
            None => ()
        }
        menu.push(String::from("! remove"));
        menu.push(String::from("* cancel"));
        match Rofi::from(rofi_config).msg(todos.get_content()[index].recap_str()).select_range(0,menu.len()-1).prompt("Edit").run(menu).unwrap().as_ref() {
            "✔ mark as done" => {
                let mut t = todos.remove(index);
                t.set_completed();
                done.push(t);
                return MenuStatus::MAINMENU;
            },
            "* cancel" => return MenuStatus::MAINMENU,
            "+ edit" => {
                let task = Rofi::from(rofi_config)
                            .prompt("Task")
                            .placeholder("")
                            .pretext(todos.get_content()[index].get_content().to_string())
                            .text_only()
                            .run(vec![])
                            .unwrap();
                if task.eq("") {
                    continue;
                }
                let old_task = todos.remove(index);
                match old_task.get_due() {
                    Some(date) => {
                        todos.push(Task::new_with_date(task, *date))
                    },
                    None => todos.push(Task::new(task))
                }
                continue;
            },
            "+ change date" => {
                let now = Local::now();
                match date_selector(rofi_config, NaiveDate::from_ymd(now.year(), now.month(), now.day())) {
                    Some(date) => {
                        let old_task = todos.remove(index);
                        todos.push(Task::new_with_date(old_task.content, date))
                    },
                    None => ()
                }
                continue;
            },
            "! remove date" => {
                let mut old_task = todos.remove(index);
                old_task.set_due(None);
                todos.push(old_task);
                continue;
            },
            "! remove" => {
                todos.remove(index);
                return MenuStatus::MAINMENU;
            },
            _ => return MenuStatus::EXIT
        }
    }
}

fn show_done_task_menu(rofi_config : &RofiParams, todos : &mut TaskList, done: &mut TaskList, index: usize) -> MenuStatus {
    loop {
        let menu =  vec![String::from("✔ mark as to do"),String::from("! remove"),String::from("* cancel")];
        match Rofi::from(rofi_config).msg(done.get_content()[index].recap_str()).select_range(0,menu.len()-1).prompt("Edit").run(menu).unwrap().as_ref() {
            "✔ mark as to do" => {
                let mut t = done.remove(index);
                t.set_not_completed();
                todos.push(t);
                return MenuStatus::BACK;
            },
            "* cancel" => return MenuStatus::BACK,
            "! remove" => {
                done.remove(index);
                return MenuStatus::BACK;
            },
            _ => return MenuStatus::EXIT
        }
    }
}

fn show_add_task(rofi_config : &RofiParams, todos : &mut TaskList) -> MenuStatus {
    let task = Rofi::from(rofi_config).prompt("Task").placeholder("").text_only().run(vec![]).unwrap();
    if task.eq("") {
        return MenuStatus::MAINMENU;
    }
    let menu =  vec![String::from("✔ validate"), String::from("+ add date"), String::from("* cancel")];
    match Rofi::from(rofi_config).prompt("Edit").select_range(0,menu.len()-1).run(menu).unwrap().as_ref() {
        "✔ validate" => {
            todos.push(Task::new(task));
            MenuStatus::MAINMENU
        },
        "* cancel" => MenuStatus::MAINMENU,
        "+ add date" => {
            let now = Local::now();
            match date_selector(rofi_config, NaiveDate::from_ymd(now.year(), now.month(), now.day())) {
                Some(date) => todos.push(Task::new_with_date(task, date)),
                None => ()
            }
            MenuStatus::MAINMENU
        },
        _ => MenuStatus::EXIT
    }
}

fn show_old_menu(rofi_config : &RofiParams, todos: &mut TaskList, done: &mut TaskList) -> MenuStatus {
    loop {
        let mut choices =  vec![String::from("← back"), String::from("@ exit")];
        for todo in done.get_content() {
            choices.push(todo.to_string());
        }
        match Rofi::from(rofi_config).prompt("Done").select_range(0,1).run(choices).unwrap().as_ref() {
            "← back" => return MenuStatus::BACK,
            "@ exit" => return MenuStatus::EXIT,
            "" => return MenuStatus::EXIT,
            s => {
                let index_result = done.get_content().iter().position(|x| x.to_string().eq(s));
                if let None = index_result {
                    continue
                }
                match show_done_task_menu(rofi_config, todos, done, index_result.unwrap()) {
                    MenuStatus::BACK => continue,
                    MenuStatus::EXIT => return MenuStatus::EXIT,
                    MenuStatus::MAINMENU => return MenuStatus::MAINMENU
                }
            }
        }
    }
}

fn show_main_menu(rofi_config : &RofiParams, todos : &mut TaskList, done: &mut TaskList) -> MenuStatus {
    loop {
        let mut choices = vec![String::from("+ add"), String::from("~ done"), String::from("@ exit")];
        for todo in todos.get_content() {
            choices.push(todo.to_string());
        }
        let status : MenuStatus = match Rofi::from(rofi_config).prompt("Todo").select_range(0,2).run(choices).unwrap().as_ref() {
            "+ add" => {
                show_add_task(rofi_config, todos)
            },
            "~ done" => {
                show_old_menu(rofi_config, todos, done)
            },
            "@ exit" => MenuStatus::EXIT,
            "" => MenuStatus::EXIT,
            s => {
                let index = todos.get_content().iter().position(|x| x.to_string().eq(s));
                match index {
                    Some(i) => show_task_menu(rofi_config, todos, done, i),
                    None => MenuStatus::MAINMENU
                }
            }
        };
        match status {
            MenuStatus::BACK => continue,
            MenuStatus::EXIT => return MenuStatus::EXIT,
            MenuStatus::MAINMENU => continue
        }
    }
}

fn load_config(config_file: &std::path::PathBuf, todos: &mut TaskList, old: &mut TaskList) -> Result<bool, String> {
    if !std::path::Path::new(config_file).exists() {
        save_config(config_file, &mut TaskList::new(), &mut TaskList::new()).unwrap();
    }
    if let Ok(lines) = read_lines(config_file) {
        for line in lines {
            if let Ok(linestr) = line {
                let task_result = Task::from_todotxt(String::from(linestr));
                match task_result {
                    Ok(task) => match task.completion {
                        false => todos.push(task),
                        true => old.push(task)
                    }
                    Err(_) => ()
                }
            }
        }
    }
    Ok(true)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>> where P: AsRef<std::path::Path> {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn save_config(config_file: &std::path::PathBuf, todos: &mut TaskList, old: &mut TaskList) -> Result<bool,String> {
    let mut save = String::new();
    for todo in todos.get_content() {
        save.push_str(&todo.to_todotxt());
        save.push_str("\n");
    }
    for todo in old.get_content() {
        save.push_str(&todo.to_todotxt());
        save.push_str("\n");
    }

    match fs::write(config_file, save) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string())
    }
}

fn main() {
    let mut todos = TaskList::new();
    let mut done = TaskList::new();

    let args = Cli::from_args();

    let sort  = match args.sort.as_ref() {
        "content" => SortTaskBy::Content,
        "creation" => SortTaskBy::CreationDate,
        "priority" => SortTaskBy::Priority,
        _ => SortTaskBy::Content
    };
    todos.change_sort(sort.clone());
    done.change_sort(sort);

    let rofi_config = RofiParams { no_config : args.no_config, case_insensitive : args.case_insensitive };

    let config = args.config;
    match load_config(&config, &mut todos, &mut done) {
        Ok(_) => (),
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    loop {
        if show_main_menu(&rofi_config, &mut todos, &mut done) == MenuStatus::EXIT { break }
    }

    match save_config(&config, &mut todos, &mut done) {
        Ok(_) => (),
        Err(s) => println!("{}", s)
    };
}
