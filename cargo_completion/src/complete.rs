
use shell_completion::{BashCompletionInput, CompletionInput, CompletionSet};
use std::process::Command;
use serde_json;

#[derive(Clone, Copy, Eq, PartialEq)]
enum ArgType {
    Lib, Bin, Example, Integration, Test, Benchmark, Build, Target, Color, Features, ErrorFmt,
    MetaVersion, Profile, Year, Vcs, Crate, Spec, Registry, Path, File, Manifest, Void, None
}

struct Arg {
    name: String,
    param: ArgType,
}

fn get_targets(wanted: Vec<ArgType>, input: &impl CompletionInput) -> Vec<String> {
    let output = Command::new("cargo")
            .arg("metadata")
            .arg("--format-version")
            .arg("1")
            .output()
            .expect("failed to execute cargo");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("metadata is not json");
    let mut tmp: Vec<String> = v.as_object().expect("json fmt error")
        .get("packages").expect("")
        .as_array().expect("")
        .into_iter().flat_map(|v: &serde_json::Value| {
            let p = v.as_object().expect("");
            let mut v = vec![];
            if wanted.contains(&ArgType::Spec) {
                v.push(p.get("name").expect("").as_str().expect("").to_string());
            }
            if wanted.contains(&ArgType::Features) {
                let mut fe = p.get("features").expect("").as_object().expect("").keys()
                    .map(|s| s.to_owned()).collect();
                v.append(&mut fe);
            }
            p.get("targets").expect("")
            .as_array().expect("").into_iter()
            .map(|v: &serde_json::Value| {
                let tmp = v.as_object().expect("");
                if tmp.get("kind").expect("").as_array().expect("")
                        .into_iter().map(|v: &serde_json::Value| match v.as_str().expect("") {
                            "bin" => ArgType::Bin,
                            "lib" | "rlib" | "dylib" | "proc-macro" => ArgType::Lib,
                            "example" => ArgType::Example,
                            "test" => ArgType::Integration,
                            "bench" => ArgType::Benchmark,
                            "custom-build" => ArgType::Build,
                            _ => panic!(""),
                        }).any(|a| wanted.contains(&a)) {
                    tmp.get("name").expect("").as_str().expect("").to_owned()
                }else {
                    "".to_owned()
                }
            }).filter(|s| s != "").chain(v.into_iter())
        }).collect();
    if wanted.contains(&ArgType::Crate) {
        let output = Command::new("cargo")
            .arg("search")
            .arg("--limit 100")
            .arg(input.current_word())
            .output()
            .expect("failed to execute cargo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut cr: Vec<String> = stdout.lines().map(|s| s.split_whitespace().next().unwrap().to_string()).collect();
        tmp.append(&mut cr);
    }
    if wanted.contains(&ArgType::Path) || wanted.contains(&ArgType::Manifest) {
        let mut pa: Vec<String> = input.complete_directory().into_iter().collect();
        tmp.append(&mut pa);
    }
    if wanted.contains(&ArgType::File) {
        let mut pa: Vec<String> = input.complete_file().into_iter().collect();
        tmp.append(&mut pa);
    }
    if wanted.contains(&ArgType::Manifest) {
        let mut pa: Vec<String> = input.complete_file().into_iter().filter(|s| s.ends_with("Cargo.toml")).collect();
        tmp.append(&mut pa);
    }
    if wanted.contains(&ArgType::Color) {
        let mut c: Vec<String> = vec!["auto".to_owned(), "always".to_owned(), "never".to_owned()];
        tmp.append(&mut c);
    }
    if wanted.contains(&ArgType::Profile) {
        let mut c: Vec<String> = vec!["test".to_owned()];
        tmp.append(&mut c);
    }
    if wanted.contains(&ArgType::MetaVersion) {
        let mut c: Vec<String> = vec!["1".to_owned()];
        tmp.append(&mut c);
    }
    if wanted.contains(&ArgType::Year) {
        let mut c: Vec<String> = vec!["2018".to_owned(), "2015".to_owned()];
        tmp.append(&mut c);
    }
    if wanted.contains(&ArgType::Vcs) {
        let mut c: Vec<String> = vec!["none".to_owned(), "git".to_owned(), "hg".to_owned(),
            "pijul".to_owned(), "fossil".to_owned()];
        tmp.append(&mut c);
    }
    if wanted.contains(&ArgType::ErrorFmt) {
        let mut c: Vec<String> = vec!["human".to_owned(), "short".to_owned(), "json".to_owned(),
            "json-diagnostic-short".to_owned(), "json-diagnostic-rendered-ansi".to_owned(),
            "json-render-diagnostics".to_owned()];
        tmp.append(&mut c);
    }
    if wanted.contains(&ArgType::Target) {
        let output = Command::new("rustc")
            .arg("--print")
            .arg("target-list")
            .output()
            .expect("failed to execute rustc");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ta: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
        tmp.append(&mut ta);
    }
    if wanted.contains(&ArgType::Test) {
        let output = Command::new("cargo")
            .arg("test")
            .arg("--")
            .arg("--list")
            .output()
            .expect("failed to execute cargo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut ta: Vec<String> = stdout.lines().take_while(|&s| s != "")
            .map(|s| s.split(":").next().unwrap().to_string()).collect();
        tmp.append(&mut ta);
    }
    tmp
}
fn arg_type(name: Option<&str>, kind: Option<&str>) -> ArgType {
    match kind {
        Some("<BENCHNAME>") => ArgType::Benchmark,
        Some("<query>") | Some("<crate>") => ArgType::Crate,
        Some("<MANIFEST>") => ArgType::Manifest,
        Some("<path>") | Some("<PATH>") => match name {
            Some("--manifest-path") => ArgType::Manifest,
            _ => ArgType::Path,
        }
        Some("<DIRECTORY>") | Some("<DIR>") => ArgType::Path,
        Some("<TESTNAME>") => ArgType::Test,
        Some("<FEATURES>") => ArgType::Features,
        Some("<NAME>") => match name {
            Some("--bin") => ArgType::Bin,
            Some("--example") => ArgType::Example,
            Some("--test") => ArgType::Integration,
            Some("--profile") => ArgType::Profile,
            Some("--name") => ArgType::Void,
            _ => ArgType::None,
        },
        Some("<VERSION>") => ArgType::MetaVersion,
        Some("<TRIPLE>") => ArgType::Target,
        Some("<SPEC>") | Some("<spec>") => ArgType::Spec,
        Some("<WHEN>") => ArgType::Color,
        Some("<FMT>") => ArgType::ErrorFmt,
        Some("<YEAR>") => ArgType::Year,
        Some("<VCS>") => ArgType::Vcs,
        Some("<REGISTRY>") => ArgType::Void,
        Some("<N>") | Some("<PRECISE>") | Some("<SHA>") | Some("<TAG>")
        | Some("<BRANCH>") | Some("<URL>") | Some("<token>") | Some("<TOKEN>")
        | Some("<LOGIN>") | Some("<INDEX>") => ArgType::Void,
        _ => ArgType::None,
    }
}

fn arg_line(line: &str) -> Vec<Arg> {
    let tmp: Vec<&str> = line.split_whitespace().take_while(|p| p.starts_with("-") || p.starts_with("<")).collect();
    let param = tmp.iter().find_map(|p| if p.starts_with("<") {
            Some(p.trim_matches(|c| c == '.' || c == ','))
        }else {
            None
        });
    tmp.iter().filter(|p| p.starts_with("-"))
        .map(|p| Arg {
            name: p.trim_matches(|c| c == '.' || c == ',').to_owned(),
            param: arg_type(Some(p.trim_matches(|c| c == '.' || c == ',')), param),
        }).collect()
}

pub fn complete_any(input: impl CompletionInput) -> Vec<String> {
    if input.current_word().ends_with(">") {
        vec![">".to_owned()]
    }else if input.previous_word().ends_with(">") {
        input.complete_file()
    }else if input.args()[0..input.arg_index()].into_iter().any(|s| s == &"--") {
        vec![]
    }else {
        let output = Command::new("cargo")
                .arg(input.args()[1])
                .arg("--help")
                .output()
                .expect("failed to execute cargo");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines().skip_while(|s| !s.starts_with("OPTIONS:") && !s.starts_with("FLAGS:"));
        let opts: Vec<Arg> = lines.by_ref().take_while(|s| s.starts_with("\t")
                    || s.starts_with("  ") || s.starts_with("OPTIONS:"))
            .flat_map(|line| arg_line(line).into_iter()).collect();
        let args: Vec<ArgType> = lines.by_ref().filter(|s| !s.starts_with("ARGS:"))
            .take_while(|s| s.trim().starts_with("<"))
            .map(|line| arg_type(None, line.split_whitespace().filter(|p| p.starts_with("<")).next()))
            .filter(|&a| a != ArgType::None).collect();
        let last = match input.args()[1] {
            "install" => "",
            "uninstall" => "",
            _ => input.previous_word(),
        };
        if let Some(a) = opts.iter().find(|a| a.name == last) {
            get_targets(vec![a.param], &input).into_iter()
                .filter(|s| s.starts_with(input.current_word())).collect()    
        }else {
            opts.iter().map(|a| a.name.to_string()).chain(get_targets(args, &input).into_iter())
                .filter(|s| s.starts_with(input.current_word())).collect()
        }
    }
}