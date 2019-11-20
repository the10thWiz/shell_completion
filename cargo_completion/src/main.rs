
mod complete;
use shell_completion::{BashCompletionInput, CompletionInput, CompletionSet};
use std::process::Command;

fn main() {
    let input = BashCompletionInput::from_env()
        .expect("Missing expected environment variables");

    complete(input).suggest();
}

fn complete(input: impl CompletionInput) -> Vec<String> {
    match input.arg_index() {
        0 => unreachable!(),
        1 => complete_cargo_commands(input),
        _ => complete::complete_any(input),
    }
}

fn complete_cargo_commands(input: impl CompletionInput) -> Vec<String> {
    let output = Command::new("cargo")
            .arg("--list")
            .output()
            .expect("failed to execute cargo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let cargo_commands : Vec<&str> = stdout.lines()
        .skip(1) // first line is description
        .map(|line| line.split_whitespace().next().unwrap()) // each line is COMMAND DESCRIPTION
        .collect();
    input.complete_subcommand(cargo_commands)
}

#[test]
fn test() {
    panic!("Intentially fails");
}

#[test]
fn test_sdgas() {
    panic!("Intentially fails");
}
