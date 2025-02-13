mod banner;

use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use owo_colors::OwoColorize;
use crate::banner::print_banner;

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

    #[cfg(feature = "sha256")]
    let checksum_calculator="sha256";
    #[cfg(not(feature = "sha256"))]
    let checksum_calculator="crc32";

    print_banner();
    println!("Using {} for checksums", checksum_calculator.cyan().bold());
    println!("Old folder: {}", args.old_folder.cyan().bold());
    println!("New folder: {}", args.new_folder.cyan().bold());
    println!("\n");

    let old_files = search_path_for_files_recursively(&args.old_folder, args.max_depth);
    let new_files = search_path_for_files_recursively(&args.new_folder, args.max_depth);

    let mut old_map = BTreeMap::new();

    let old_shasums = create_checksums(&old_files, "Calculating shasums for old files".to_owned());
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

    let new_shasums: Vec<String> =
        create_checksums(&new_files, "Calculating shasums for new files".to_owned());
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

#[cfg(not(feature = "sha256"))]
fn create_checksum(d: &PathBuf) -> String {
    let data = fs::read(d).unwrap();
    crc32fast::hash(data.as_slice()).to_string()
}

#[cfg(feature = "sha256")]
fn create_checksum(d: &PathBuf) -> String {
    sha256::try_digest(d.as_path()).unwrap()
}

fn create_checksums(old_files: &Vec<PathBuf>, message: String) -> Vec<String> {
    let style = ProgressStyle::with_template(
        "{msg}: {wide_bar:.cyan/blue} {pos:>7}/{len} ({per_sec})",
    )
    .unwrap();

    let old_shasums: Vec<String> = old_files
        .par_iter()
        .progress_with_style(style)
        .with_message(message)
        .map(create_checksum)
        .collect();
    old_shasums
}
