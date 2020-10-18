use std::io::prelude::*;
use std::path::PathBuf;

use yaml_rust::Yaml;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Ujjwal Sharma <ryzokuken@disroot.org>")]
struct Opts {
    #[clap(long)]
    root_path: PathBuf,
}

fn extract_strings(yaml: Option<&Yaml>) -> Option<Vec<String>> {
    match yaml {
        Some(Yaml::Array(array)) => Some(
            array
                .iter()
                .map(|v| match v {
                    Yaml::String(s) => s.clone(),
                    _ => String::new(),
                })
                .collect(),
        ),
        _ => None,
    }
}

fn process_file(test_path: &PathBuf, root_path: &PathBuf) {
    let mut file = std::fs::File::open(test_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let yaml_start = contents.find("/*---");
    if let Some(start) = yaml_start {
        let yaml_end = contents.find("---*/");
        let frontmatter = yaml_rust::YamlLoader::load_from_str(
            contents.get(start + 6..yaml_end.unwrap()).unwrap(),
        )
        .unwrap();
        if let Yaml::Hash(h) = &frontmatter[0] {
            // let flags = extract_strings(h.get(&Yaml::String(String::from("flags"))));
            // let features = extract_strings(h.get(&Yaml::String(String::from("features"))));
            let mut includes =
                extract_strings(h.get(&Yaml::String(String::from("includes")))).unwrap_or_default();
            includes.push(String::from("assert.js"));
            includes.push(String::from("sta.js"));
            let include_path = root_path.join("harness");
            let mut include_contents = String::new();
            for include in includes {
                let mut include_file = std::fs::File::open(include_path.join(include)).unwrap();
                let mut include_file_contents = String::new();
                include_file
                    .read_to_string(&mut include_file_contents)
                    .unwrap();
                include_contents.push_str(include_file_contents.as_ref());
                include_contents.push('\n');
            }
        }
    }
}

fn main() {
    // TODO: add CLI option for test_path
    // TODO: add CLI option for include_path
    // test_path ||= root_path + 'test'
    // include_path ||= root_path + 'harness'
    // TODO: if test_path points to a file, call process_file directly
    let args = Opts::parse();
    let root_path = args.root_path;
    let test_path = root_path.join("test");
    for entry in walkdir::WalkDir::new(test_path) {
        let ent = entry.unwrap();
        if ent.metadata().unwrap().is_file() {
            process_file(&ent.into_path(), &root_path);
        }
    }
}
