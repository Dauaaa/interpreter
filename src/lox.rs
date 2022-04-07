use anyhow::Result;
use std::{io::Write, path::Path};

mod error;
mod scanner;
mod tokens;

use scanner::Scanner;

pub struct Lox {
    pub has_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Lox { has_error: false }
    }
}

impl Lox {
    pub fn run_prompt() -> Result<i32> {
        loop {
            print!("> ");
            std::io::stdout().flush().unwrap();

            let mut code = String::new();

            std::io::stdin().read_line(&mut code)?;

            match &code.lines().next().unwrap() {
                x if x.len() == 0 => {
                    continue;
                }
                code => {
                    let lox = Lox::new();
                    lox.run(code);
                }
            }
        }
    }
    pub fn run_file(file: &Path) -> Result<i32> {
        let code = std::fs::read_to_string(file)?;

        let lox = Lox::new();

        lox.run(&code);

        Ok(0)
    }
    pub fn run(&self, code: &str) {
        let mut scanner = Scanner::new([code, "\n"].concat());
        println!("{:?}", scanner.scan_tokens());
    }
}
