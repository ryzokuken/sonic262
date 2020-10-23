use clap::Clap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Ujjwal Sharma <ryzokuken@disroot.org>")]
struct Opts {
    #[clap(long)]
    file_to_test: PathBuf,
}

// TODO maybe use a logging crate instead of doing... this
fn main() {
    let args = Opts::parse();
    let file_to_test = args.file_to_test;
    let frontmatter = extract_frontmatter(&file_to_test);
    match frontmatter {
        Some(f) => match get_include_files(
            &f,
            PathBuf::from("/home/humancalico/code/test262/harness"),
        ) {
            Ok(include_files) => match generate_final_file_to_test(&file_to_test, include_files) {
                Ok(file_to_run) => run_file_in_node(file_to_run),
                Err(e) => eprintln!(
                    "Couldn't generate file: {:?} to test. Err: {}",
                    file_to_test, e
                ),
            },
            Err(e) => eprintln!("No includes for the file {:?}, Err: {}", file_to_test, e),
        },
        None => eprintln!(
            "Couldn't convert frontmatter of file: {:?} to serde_yaml::Value",
            file_to_test
        ),
    }
}

// fn walk(root_path: PathBuf) -> Result<PathBuf> {
// TODO walk file in parallel using jwalk
// }

// TODO use a Result here instead of an Option
// TODO do this using the FromStr trait maybe
fn extract_frontmatter(file_to_test: &PathBuf) -> Option<String> {
    // FIXME remove unwrap
    // TODO Read asynchronously
    let file_contents = fs::read_to_string(file_to_test).unwrap();
    // TODO cleanup using the and_then method
    let yaml_start = file_contents.find("/*---");
    if let Some(start) = yaml_start {
        let yaml_end = file_contents.find("---*/");
        if let Some(end) = yaml_end {
            // TODO remove unwrap here
            Some(file_contents.get(start + 5..end).unwrap().to_string())
        } else {
            eprintln!("This file has an invalid frontmatter");
            None
        }
    } else {
        eprintln!("frontmatter not found in file: {:?}", file_to_test);
        None
    }
}

// FIXME sorry about the naming here
fn get_include_files(
    frontmatter_str: &str,
    include_path_root: PathBuf,
) -> serde_yaml::Result<Vec<PathBuf>> {
    let frontmatter_value: serde_yaml::Value = serde_yaml::from_str(frontmatter_str)?;
    // TODO use .ok_or_else() here
    let includes_yaml = frontmatter_value.get("includes").unwrap();
    // TODO use turbofish like .collect()
    let mut includes: Vec<String> = serde_yaml::from_value(includes_yaml.clone())?;
    let must_include = &mut vec!["assert.js".to_string(), "sta.js".to_string()];
    includes.append(must_include);
    let mut include_paths: Vec<PathBuf> = vec![];
    for include in includes {
        include_paths.push(include_path_root.join(include));
    }
    Ok(include_paths)
}

fn generate_final_file_to_test(
    file_to_test: &PathBuf,
    include_files_to_add: Vec<PathBuf>,
) -> std::io::Result<NamedTempFile> {
    // TODO do this asynchronously
    let mut contents = fs::read_to_string(file_to_test)?;
    for file in include_files_to_add {
        let file_contents = fs::read_to_string(file)?;
        contents.push_str(&file_contents);
    }
    let mut file = tempfile::Builder::new().suffix(".js").tempfile()?;
    writeln!(file, "{}", contents)?;
    Ok(file)
}

fn run_file_in_node(file: NamedTempFile) {
    // TODO remove use of unwrap here
    let _node_process = Command::new("node").arg(file.path()).output().unwrap();
}
