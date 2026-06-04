use std::collections::BTreeMap;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChecksumGenerator {
    #[default]
    CRC32,
    #[cfg(feature = "sha256")]
    Sha256,
    #[cfg(feature = "adler32")]
    Adler32,
    #[cfg(feature = "md5")]
    MD5,
}

pub fn search_path_for_files_recursively(path: &String, max_depth: i16) -> Vec<PathBuf> {
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

pub fn get_generator_name(generator: &ChecksumGenerator) -> String {
    match generator {
        ChecksumGenerator::CRC32 => "CRC 32".to_owned(),
        #[cfg(feature = "sha256")]
        ChecksumGenerator::Sha256 => "Sha256".to_owned(),
        #[cfg(feature = "adler32")]
        ChecksumGenerator::Adler32 => "Adler 32".to_owned(),
        #[cfg(feature = "md5")]
        ChecksumGenerator::MD5 => "MD5".to_owned(),
    }
}

fn create_checksum_crc32(d: &PathBuf) -> String {
    let data = fs::read(d).unwrap();
    crc32fast::hash(data.as_slice()).to_string()
}

#[cfg(feature = "sha256")]
fn create_checksum_sha256(d: &PathBuf) -> String {
    sha256::try_digest(d.as_path()).unwrap()
}

#[cfg(feature = "adler32")]
fn create_checksum_adler32(d: &PathBuf) -> String {
    let reader = fs::File::open(d).unwrap();
    adler32::adler32(reader).unwrap().to_string()
}

#[cfg(feature = "md5")]
fn create_checksum_md5(d: &PathBuf) -> String {
    let data = fs::read(d).unwrap();
    let string_vec: Vec<String> = md5::compute(data.as_slice()).map(|x| x.to_string()).into();
    string_vec.join("")
}

fn get_checksum_function(generator: &ChecksumGenerator) -> for<'a> fn(&'a PathBuf) -> String {
    match generator {
        ChecksumGenerator::CRC32 => create_checksum_crc32,
        #[cfg(feature = "sha256")]
        ChecksumGenerator::Sha256 => create_checksum_sha256,
        #[cfg(feature = "adler32")]
        ChecksumGenerator::Adler32 => create_checksum_adler32,
        #[cfg(feature = "md5")]
        ChecksumGenerator::MD5 => create_checksum_md5,
    }
}

pub fn create_checksums(
    files: &Vec<PathBuf>,
    message: String,
    generator: &ChecksumGenerator,
    console_progress: bool,
) -> Vec<String> {
    let style =
        ProgressStyle::with_template("{msg}: {wide_bar:.cyan/blue} {pos:>7}/{len} ({per_sec})")
            .unwrap();

    let check_fn = get_checksum_function(generator);

    if console_progress {
        files
            .par_iter()
            .progress_with_style(style)
            .with_message(message)
            .map(check_fn)
            .collect()
    } else {
        files.par_iter().map(check_fn).collect()
    }
}

pub fn check_files(old_folder: &String, new_folder: &String, max_depth: i16, generator: &ChecksumGenerator, console_progress: bool) -> Vec<String> {
    let old_files = search_path_for_files_recursively(old_folder, max_depth);
    let new_files = search_path_for_files_recursively(new_folder, max_depth);

    let mut old_map = BTreeMap::new();

    let old_shasums = create_checksums(&old_files, "Calculating checksums for old files".to_owned(), generator, console_progress);
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
        create_checksums(&new_files, "Calculating checksums for new files".to_owned(), generator, console_progress);
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
            missing_files.push(path.to_owned());
        }
    }
    missing_files
}