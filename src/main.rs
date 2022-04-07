mod lox;

use lox::Lox;
use std::{env, path::Path, process};
fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            println!("wtf");
            Lox::run_prompt().unwrap();
        }
        2 => {
            Lox::run_file(Path::new(&args[0])).unwrap();
        }
        _ => {
            println!("Usage: jlox [script]");
            process::exit(64);
        }
    }
    process::exit(0);
}
