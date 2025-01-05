use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sha256::try_digest;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    old_folder: String,

    #[arg(short, long)]
    new_folder: String,

    #[arg(short = 'd', long, default_value = "1000")]
    max_depth: i16,
}

fn search_path_for_files_recursively(path: &String, max_depth: i16) -> Vec<PathBuf> {
    let mut results: Vec<PathBuf> = Vec::new();
    if max_depth <= 0 {
        return results;
    }

    let paths = fs::read_dir(path).unwrap();
    for path in paths {
        let entry = path.unwrap();
        if entry.file_type().unwrap().is_file() {
            results.push(entry.path());
        } else if entry.file_type().unwrap().is_dir() {
            let mut sub_results = search_path_for_files_recursively(
                &entry.path().to_str().unwrap().to_string(),
                max_depth - 1,
            );
            results.append(&mut sub_results);
        }
    }

    results
}

fn main() {
    let args = Args::parse();

    let old_files = search_path_for_files_recursively(&args.old_folder, args.max_depth);
    let new_files = search_path_for_files_recursively(&args.new_folder, args.max_depth);

    println!("Calculating shasums for old and new files...");
    let mut old_map = BTreeMap::new();

    let old_shasums = create_shasums(&old_files);
    for i in 0..old_files.len() {
        old_map.insert(
            old_files
                .get(i)
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
                .to_string(),
            old_shasums.get(i).unwrap(),
        );
    }

    let mut new_map = BTreeMap::new();

    let new_shasums: Vec<String> = create_shasums(&new_files);
    for i in 0..new_files.len() {
        new_map.insert(
            new_shasums.get(i).unwrap(),
            new_files
                .get(i)
                .unwrap()
                .as_path()
                .to_str()
                .unwrap()
                .to_string(),
        );
    }

    let mut missing_files = Vec::new();
    for (path, shasum) in old_map.iter() {
        if !new_map.contains_key(shasum) {
            missing_files.push(path);
        }
    }

    println!("Missing files: {:#?}", missing_files);
}

fn create_shasums(old_files: &Vec<PathBuf>) -> Vec<String> {
    let old_shasums: Vec<String> = old_files
        .par_iter()
        .progress_count(old_files.len() as u64)
        .map(|d| try_digest(d.as_path()).unwrap())
        .collect();
    old_shasums
}
