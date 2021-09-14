use std::process::Command;
use std::process::Stdio;
use std::io::Write;

pub struct RofiParams {
    pub no_config: bool,
    pub case_insensitive: bool
}

pub struct Rofi {
    rofi: Command,
}

impl Rofi {
    pub fn new() -> Self {
        let mut r = Rofi { rofi : Command::new("rofi") };
        r.rofi.arg("-dmenu");
        r
    }

    pub fn from(p : &RofiParams) -> Self {
        let mut rofi = Self::new();
        if p.no_config {
            rofi = rofi.no_config();
        }
        if p.case_insensitive {
            rofi = rofi.case_insensitive();
        }
        rofi
    }

    pub fn run(mut self, entries: Vec<String>) -> Result<String, String> {
        let mut proc = self.rofi.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()          
            .ok()
            .expect("failed to spawn rust");


        let entry_list = str2bytes(entries);
        proc.stdin.as_mut().unwrap().write_all(entry_list.as_bytes()).expect("Erreur avec stdin");

        let mut retour = String::from_utf8(proc.wait_with_output().unwrap().stdout).unwrap();
        trim_newline(&mut retour);
        Ok(retour)
    }

    pub fn msg(mut self, m: String) -> Self {
        self.rofi.arg("-mesg").arg(m);
        self
    }

    pub fn no_config(mut self) -> Self {
        self.rofi.arg("-no-config");
        self
    }

    pub fn case_insensitive(mut self) -> Self {
        self.rofi.arg("-i");
        self
    }

    pub fn prompt(mut self, p: &str) -> Self {
        self.rofi.arg("-p")
            .arg(p);
        self
    }

    pub fn selected(mut self, index: u32) -> Self {
        self.rofi.arg("-selected-row")
            .arg(index.to_string());
        self
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.rofi.arg("-theme-str")
            .arg(format!("entry {{ placeholder: \"{}\"; }}",placeholder ));
        self
    }

    pub fn text_only(mut self) -> Self {
        self.rofi.arg("-l").arg("0");
        self
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

fn str2bytes(tab : Vec<String>) -> String {
    let mut s = String::new();
    for x in tab {
        s.push_str(&x);
        s.push('\n');
    }
    s
}