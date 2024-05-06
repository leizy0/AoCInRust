use std::{
    collections::{HashMap, VecDeque},
    env, error,
    fmt::Display,
    hash::Hash,
    io::{self, stdin, stdout, Write},
    iter,
};

use regex::Regex;

use crate::int_code::io::{InputPort, OutputPort};

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    WrongNumberOfArgs(usize, usize), // (number of given arguments, expected number of arguments),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IOError(ioe) => write!(f, "I/O error: {}", ioe),
            Error::WrongNumberOfArgs(real_n, expect_n) => write!(
                f,
                "Given wrong number({}) of arguemnts, expect {}",
                real_n, expect_n
            ),
        }
    }
}

impl error::Error for Error {}

#[derive(Debug)]
struct RegexWrapper {
    regex_str: String,
    regex: Regex,
}

impl RegexWrapper {
    pub fn new(regex_str: &str) -> Self {
        Self {
            regex_str: regex_str.to_string(),
            regex: Regex::new(regex_str).unwrap(),
        }
    }

    pub fn is_match(&self, target_str: &str) -> bool {
        self.regex.is_match(target_str)
    }
}

impl PartialEq for RegexWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.regex_str == other.regex_str
    }
}

impl Eq for RegexWrapper {}

impl Hash for RegexWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.regex_str.hash(state);
    }
}

pub struct DroidConsole {
    input_buf: VecDeque<char>,
    output_buf: String,
    log: String,
    com_patterns: HashMap<RegexWrapper, Option<String>>, // <command pattern in regular expression, candidate complete command string>
    is_end: bool,
}

impl DroidConsole {
    pub fn new() -> Self {
        let com_patterns = Self::init_coms();
        Self {
            input_buf: VecDeque::new(),
            output_buf: String::new(),
            log: String::new(),
            com_patterns,
            is_end: false,
        }
    }

    fn req_command(&mut self) {
        let mut buf = String::new();
        loop {
            buf.clear();
            print!("DC> ");
            stdout().flush().unwrap();
            if let Ok(_) = stdin()
                .read_line(&mut buf)
                .inspect_err(|e| eprintln!("Failed to read input from console, get error({}).", e))
            {
                let trimmed_buf = buf.trim_end();
                if trimmed_buf.is_empty() {
                    continue;
                } else if let Some(command) = self
                    .com_patterns
                    .iter()
                    .flat_map(|(k, v)| {
                        v.as_ref()
                            .map(|s| s.as_str())
                            .or(Some(trimmed_buf))
                            .filter(|_| k.is_match(trimmed_buf))
                    })
                    .next()
                {
                    // Found first matched command pattern.
                    self.input_buf
                        .extend(command.chars().chain(iter::once('\n')));
                    if command == "exit" {
                        self.is_end = true;
                    }
                    break;
                } else {
                    Self::print_usage();
                }
            }
        }
    }

    fn init_coms() -> HashMap<RegexWrapper, Option<String>> {
        HashMap::from([
            (RegexWrapper::new(r"^n|(north)$"), Some("north".to_string())),
            (RegexWrapper::new(r"^s|(south)$"), Some("south".to_string())),
            (RegexWrapper::new(r"^w|(west)$"), Some("west".to_string())),
            (RegexWrapper::new(r"^e|(east)$"), Some("east".to_string())),
            (RegexWrapper::new(r"^take .+$"), None),
            (RegexWrapper::new(r"^drop .+$"), None),
            (RegexWrapper::new(r"^i|(inv)$"), Some("inv".to_string())),
            (RegexWrapper::new(r"^x|(exit)$"), Some("exit".to_string())),
        ])
    }

    fn print_usage() {
        println!("Usage: n[orth], s[outh], w[est], e[ast] for moving; take {{object name}} for picking up; drop {{object name}} for dropping; i[nv] for listing items; [e]x[it] for exit.")
    }
}

impl InputPort for DroidConsole {
    fn get(&mut self) -> Option<i64> {
        if self.input_buf.is_empty() {
            self.req_command();
        }

        if self.is_end {
            None
        } else {
            self.input_buf.pop_front().map(|c| {
                // Echo and log input.
                print!("{}", c);
                self.log.push(c);
                c as i64
            })
        }
    }

    fn reg_proc(&mut self, _proc_id: usize) {}
}

impl OutputPort for DroidConsole {
    fn put(&mut self, value: i64) -> Result<(), crate::Error> {
        if let Some(c) = u32::try_from(value).ok().and_then(|u| char::from_u32(u)) {
            self.output_buf.push(c);
            if c == '\n' {
                print!("{}", self.output_buf);
                self.log.push_str(&self.output_buf);
                self.output_buf.clear();
            }
        }

        Ok(())
    }

    fn wait_proc_id(&self) -> Option<usize> {
        None
    }
}

pub fn check_args() -> Result<String, Error> {
    let args = env::args();
    let args_n = args.len();
    if args_n != 2 {
        Err(Error::WrongNumberOfArgs(args_n, 2))
    } else {
        Ok(args.skip(1).next().unwrap().to_string())
    }
}
