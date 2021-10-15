mod rofi;
use rofi::{Rofi, RofiParams};
mod task;
use task::{Task, SortTaskBy};
mod date_selector;
use date_selector::date_selector;
use std::fs;
use std::io::{self, BufRead};
use structopt::StructOpt;
use chrono::{Local, NaiveDate, Datelike};
mod indexer;
use indexer::Indexer;
use std::rc::Rc;

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
    #[structopt(short = "s", long="sort", possible_values = &["creation","content","priority","due"], case_insensitive = true, default_value="content")]
    sort : String
}

#[derive(PartialEq)]
enum MenuStatus {
    EXIT,
    MAINMENU,
    BACK
}

fn show_task_menu(rofi_config : &RofiParams, params : &mut Params, task: Rc<Task>) -> MenuStatus {
    let mut updated_task = task;
    loop {
        let mut menu =  vec![String::from("✔ mark as done"), String::from("+ edit"), String::from("+ change date")];
        match updated_task.get_due() {
            Some(_) => menu.push(String::from("! remove date")),
            None => ()
        }
        menu.push(String::from("! remove"));
        menu.push(String::from("* cancel"));
        match Rofi::from(rofi_config).msg(updated_task.recap_str()).select_range(0,menu.len()-1).prompt("Edit").run(menu).unwrap().as_ref() {
            "✔ mark as done" => {
                let mut t = params.todos.remove(updated_task).expect("Some references to task were not deleted");
                t.set_completed();
                add_task(&mut params.todos,t);
                return MenuStatus::BACK;
            },
            "* cancel" => return MenuStatus::BACK,
            "+ edit" => {
                let task = Rofi::from(rofi_config)
                            .prompt("Task")
                            .placeholder("")
                            .pretext(updated_task.get_content().to_string())
                            .text_only()
                            .run(vec![])
                            .unwrap();
                if task.eq("") {
                    continue;
                }
                let mut old_task = params.todos.remove(updated_task).expect("Some references to task were not deleted");
                old_task.set_content(task);
                updated_task = add_task(&mut params.todos,old_task);
                continue;
            },
            "+ change date" => {
                let now = Local::now();
                match date_selector(rofi_config, NaiveDate::from_ymd(now.year(), now.month(), now.day())) {
                    Some(date) => {
                        let old_task = params.todos.remove(updated_task).expect("Some references to task were not deleted");
                        updated_task = add_task(&mut params.todos,Task::new_with_date(old_task.content, date));
                    },
                    None => ()
                }
                continue;
            },
            "! remove date" => {
                let mut old_task = params.todos.remove(updated_task).expect("Some references to task were not deleted");
                old_task.set_due(None);
                updated_task = add_task(&mut params.todos,old_task);
                continue;
            },
            "! remove" => {
                params.todos.remove(updated_task);
                return MenuStatus::BACK;
            },
            _ => return MenuStatus::EXIT
        }
    }
}

fn show_done_task_menu(rofi_config : &RofiParams, params : &mut Params, task: Rc<Task>) -> MenuStatus {
    loop {
        let menu =  vec![String::from("✔ mark as to do"),String::from("! remove"),String::from("* cancel")];
        match Rofi::from(rofi_config).msg(task.recap_str()).select_range(0,menu.len()-1).prompt("Edit").run(menu).unwrap().as_ref() {
            "✔ mark as to do" => {
                let mut t = params.todos.remove(task).expect("Some references to task were not deleted");
                t.set_not_completed();
                add_task(&mut params.todos,t);
                return MenuStatus::BACK;
            },
            "* cancel" => return MenuStatus::BACK,
            "! remove" => {
                params.todos.remove(task);
                return MenuStatus::BACK;
            },
            _ => return MenuStatus::EXIT
        }
    }
}

fn show_add_task(rofi_config : &RofiParams, params : &mut Params) -> MenuStatus {
    let task = Rofi::from(rofi_config).prompt("Task").placeholder("").text_only().run(vec![]).unwrap();
    if task.eq("") {
        return MenuStatus::MAINMENU;
    }
    let menu =  vec![String::from("✔ validate"), String::from("+ add date"), String::from("* cancel")];
    match Rofi::from(rofi_config).prompt("Edit").select_range(0,menu.len()-1).run(menu).unwrap().as_ref() {
        "✔ validate" => {
            add_task(&mut params.todos,Task::new(task));
            MenuStatus::MAINMENU
        },
        "* cancel" => MenuStatus::MAINMENU,
        "+ add date" => {
            let now = Local::now();
            match date_selector(rofi_config, NaiveDate::from_ymd(now.year(), now.month(), now.day())) {
                Some(date) => {add_task(&mut params.todos,Task::new_with_date(task, date));},
                None => ()
            }
            MenuStatus::MAINMENU
        },
        _ => MenuStatus::EXIT
    }
}

fn show_old_menu(rofi_config : &RofiParams, params : &mut Params) -> MenuStatus {
    loop {
        let mut choices =  vec![String::from("← back"), String::from("* exit")];
        for todo in params.todos.index(&String::from("done")).unwrap() {
            choices.push(todo.to_string());
        }
        match Rofi::from(rofi_config).prompt("Done").select_range(0,1).run(choices).unwrap().as_ref() {
            "← back" => return MenuStatus::BACK,
            "* exit" => return MenuStatus::EXIT,
            "" => return MenuStatus::EXIT,
            s => {
                let result = params.todos.index(&String::from("done")).unwrap().into_iter().find(|x| x.to_string().eq(s));
                if let None = result {
                    continue
                }
                match show_done_task_menu(rofi_config, params, result.unwrap()) {
                    MenuStatus::BACK => continue,
                    MenuStatus::EXIT => return MenuStatus::EXIT,
                    MenuStatus::MAINMENU => return MenuStatus::MAINMENU
                }
            }
        }
    }
}

fn show_tags_menu(rofi_config : &RofiParams, params : &mut Params, index_name: String) -> MenuStatus {
    loop {
        let mut choices = vec![String::from("← back")];
        // Exiting if the index was removed
        let idx = match params.todos.index(&index_name) {
            Some(index) => index,
            None => return MenuStatus::BACK
        };
        for todo in idx {
            choices.push(todo.to_string());
        }
        let status : MenuStatus = match Rofi::from(rofi_config).prompt("Todo").select_range(0,0).run(choices).unwrap().as_ref() {
            "← back" => MenuStatus::MAINMENU,
            "" => MenuStatus::EXIT,
            s => {
                let result = params.todos.index(&index_name).unwrap().into_iter().find(|x| x.to_string().eq(s));
                match result {
                    Some(t) => show_task_menu(rofi_config, params, t),
                    None => MenuStatus::MAINMENU
                }
            }
        };
        match status {
            MenuStatus::BACK => continue,
            MenuStatus::EXIT => return MenuStatus::EXIT,
            MenuStatus::MAINMENU => return MenuStatus::MAINMENU
        }
    }
}

fn show_tag_list(rofi_config : &RofiParams, params : &mut Params, tag_type: String) -> MenuStatus {
    loop {
        let mut choices = vec![String::from("← back")];
        let tags = params.todos.get_index_list()
                                .iter()
                                .filter(|x|x.starts_with(&tag_type))
                                .map(|x|{let mut s = String::from(*x); s.replace_range(0..tag_type.len(), ""); s})
                                .collect::<Vec<String>>();
        for tag in tags {
            choices.push(tag.to_string());
        }
        let status : MenuStatus = match Rofi::from(rofi_config).prompt("Tag").select_range(0,0).run(choices).unwrap().as_ref() {
            "← back" => MenuStatus::MAINMENU,
            "" => MenuStatus::EXIT,
            s => {
                let mut idx_name = tag_type.to_string();
                idx_name.push_str(s);
                let result = params.todos.index(&idx_name);
                match result {
                    Some(_) => show_tags_menu(rofi_config, params, idx_name),
                    None => MenuStatus::BACK
                }
            }
        };
        match status {
            MenuStatus::BACK => continue,
            MenuStatus::EXIT => return MenuStatus::EXIT,
            MenuStatus::MAINMENU => return MenuStatus::MAINMENU
        }
    }
}

fn show_main_menu(rofi_config : &RofiParams, params : &mut Params) -> MenuStatus {
    loop {
        let mut choices = vec![String::from("+ add"), String::from("~ done"), String::from("@ project tags"), String::from("@ context tags") , String::from("* exit")];
        for todo in params.todos.index(&params.get_sort_string()).unwrap() {
            choices.push(todo.to_string());
        }
        let status : MenuStatus = match Rofi::from(rofi_config).prompt("Todo").select_range(0,4).run(choices).unwrap().as_ref() {
            "+ add" => {
                show_add_task(rofi_config, params)
            },
            "~ done" => {
                show_old_menu(rofi_config, params)
            },
            "@ project tags" => {
                show_tag_list(rofi_config, params, String::from("project_"))
            },
            "@ context tags" => {
                show_tag_list(rofi_config, params, String::from("context_"))
            },
            "* exit" => MenuStatus::EXIT,
            "" => MenuStatus::EXIT,
            s => {
                let result = params.todos.index(&params.get_sort_string()).unwrap().into_iter().find(|x| x.to_string().eq(s));
                match result {
                    Some(t) => show_task_menu(rofi_config, params, t),
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

fn load_config(config_file: &std::path::PathBuf, todos: &mut Indexer<Task>) -> Result<bool, String> {
    if !std::path::Path::new(config_file).exists() {
        save_config(config_file, todos).unwrap();
    }
    if let Ok(lines) = read_lines(config_file) {
        for line in lines {
            if let Ok(linestr) = line {
                let task_result = Task::from_todotxt(String::from(linestr));
                match task_result {
                    Ok(task) => {add_task(todos, task);}
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

fn save_config(config_file: &std::path::PathBuf, todos: &mut Indexer<Task>) -> Result<bool,String> {
    let mut save = String::new();
    for todo in todos.get_main_index() {
        save.push_str(&todo.to_todotxt());
        save.push_str("\n");
    }

    match fs::write(config_file, save) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string())
    }
}

fn add_task(idx: &mut Indexer<Task>, tsk: Task) -> Rc<Task> {
    if !tsk.completion {
        for tag in tsk.get_context_tags().clone() {
            let mut idx_name = String::from("context_");
            idx_name.push_str(&tag);
            idx.new_autoremove_index(idx_name, move |x|!x.completion && x.get_context_tags().contains(&tag), Task::comp_content);
        }
        for tag in tsk.get_project_tags().clone() {
            let mut idx_name = String::from("project_");
            idx_name.push_str(&tag);
            idx.new_autoremove_index(idx_name, move |x|!x.completion && x.get_project_tags().contains(&tag), Task::comp_content);
        }
    }
    idx.add(tsk)
}


struct Params {
    sort : SortTaskBy,
    todos : Indexer<Task>,
}

impl Params {
    fn new(sort : SortTaskBy, idx : Indexer<Task>) -> Self {
        Params { sort : sort, todos : idx }
    }

    fn get_sort_string(&self) -> String {
        String::from(match self.sort {
            SortTaskBy::Content         => "content",
            SortTaskBy::CreationDate    => "creation",
            SortTaskBy::Priority        => "priority",
            SortTaskBy::DueDate         => "due"
        })
    }
}

fn main() {
    let mut todos = Indexer::<Task>::new();

    let args = Cli::from_args();

    let sort  = match args.sort.as_ref() {
        "content"   => SortTaskBy::Content,
        "creation"  => SortTaskBy::CreationDate,
        "priority"  => SortTaskBy::Priority,
        "due"       => SortTaskBy::DueDate,
        _           => SortTaskBy::Content
    };

    todos.new_index(String::from("content"),    |x|!x.completion, Task::comp_content);
    todos.new_index(String::from("creation"),   |x|!x.completion, Task::comp_creation_date);
    todos.new_index(String::from("priority"),   |x|!x.completion, Task::comp_priority);
    todos.new_index(String::from("due"),        |x|!x.completion, Task::comp_due_date);
    todos.new_index(String::from("done"),       |x|x.completion, Task::comp_content);

    let rofi_config = RofiParams { no_config : args.no_config, case_insensitive : args.case_insensitive };
    let config = args.config;
    match load_config(&config, &mut todos) {
        Ok(_) => (),
        Err(s) => {
            println!("{}", s);
            return;
        }
    };

    let mut parameters = Params::new(sort, todos);

    loop {
        if show_main_menu(&rofi_config, &mut parameters) == MenuStatus::EXIT { break }
    }

    match save_config(&config, &mut parameters.todos) {
        Ok(_) => (),
        Err(s) => println!("{}", s)
    };
}
