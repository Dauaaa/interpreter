use colored::Colorize;
use std::iter;

pub fn report_error(line: usize, offset: usize, code: &String, message: String) {
    let (slice_back, slice_front) = (15usize, 15usize);
    let line_pos = format!("[line: {}; pos: {}]", format!("{}", line).blue(), format!("{}", offset).blue());
    println!("
    {}
    {}
    {}{}{}
    {}{}
    {}{}
    {}{}
    Error msg: {}",

    format!("ERROR").red().bold(),
    line_pos,
    format!("{}", code.clone().lines().nth(line - 1).into_iter().collect::<String>().chars().skip(offset.max(slice_front) - slice_front).take(offset.min(slice_front)).collect::<String>()).yellow(),
    format!("{}", code.clone().lines().nth(line - 1).into_iter().collect::<String>().chars().skip(offset.max(1)).take(1).collect::<String>()).red().underline(),
    format!("{}", code.clone().lines().nth(line - 1).into_iter().collect::<String>().chars().skip(offset + 1).take(slice_back).collect::<String>()).yellow(),
    iter::repeat(" ").take(offset.min(slice_front)).collect::<String>(), "^",
    iter::repeat(" ").take(offset.min(slice_front)).collect::<String>(), "|",
    iter::repeat("-").take(offset.min(slice_front)).collect::<String>(), "+",
    format!("{}", message).red().underline());
}
