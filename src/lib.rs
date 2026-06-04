use indicatif::{ParallelProgressIterator, ProgressStyle};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
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

fn create_checksum_crc32(d: &Path) -> String {
    let data = fs::read(d).unwrap();
    crc32fast::hash(data.as_slice()).to_string()
}

#[cfg(feature = "sha256")]
fn create_checksum_sha256(d: &Path) -> String {
    sha256::try_digest(d).unwrap()
}

#[cfg(feature = "adler32")]
fn create_checksum_adler32(d: &Path) -> String {
    let reader = fs::File::open(d).unwrap();
    adler32::adler32(reader).unwrap().to_string()
}

#[cfg(feature = "md5")]
fn create_checksum_md5(d: &Path) -> String {
    let data = fs::read(d).unwrap();
    let string_vec: Vec<String> = md5::compute(data.as_slice()).map(|x| x.to_string()).into();
    string_vec.join("")
}

fn get_checksum_function(generator: &ChecksumGenerator) -> for<'a> fn(&'a Path) -> String {
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
            .map(|path| check_fn(path.as_path()))
            .collect()
    } else {
        files
            .par_iter()
            .map(|path| check_fn(path.as_path()))
            .collect()
    }
}

pub fn find_duplicate_files(
    folder: &String,
    max_depth: i16,
    generator: &ChecksumGenerator,
    console_progress: bool,
) -> Vec<Vec<String>> {
    let files = search_path_for_files_recursively(folder, max_depth);
    let checksums = create_checksums(
        &files,
        "Calculating checksums for duplicate scan".to_owned(),
        generator,
        console_progress,
    );

    let mut checksum_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (file, checksum) in files.iter().zip(checksums.iter()) {
        checksum_map
            .entry(checksum.to_owned())
            .or_default()
            .push(file.as_path().to_str().unwrap().to_string());
    }

    checksum_map
        .into_values()
        .filter_map(|mut paths| {
            if paths.len() > 1 {
                paths.sort();
                Some(paths)
            } else {
                None
            }
        })
        .collect()
}

pub fn check_files(
    old_folder: &String,
    new_folder: &String,
    max_depth: i16,
    generator: &ChecksumGenerator,
    console_progress: bool,
) -> Vec<String> {
    let old_files = search_path_for_files_recursively(old_folder, max_depth);
    let new_files = search_path_for_files_recursively(new_folder, max_depth);

    let mut old_map = BTreeMap::new();

    let old_shasums = create_checksums(
        &old_files,
        "Calculating checksums for old files".to_owned(),
        generator,
        console_progress,
    );
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

    let new_shasums: Vec<String> = create_checksums(
        &new_files,
        "Calculating checksums for new files".to_owned(),
        generator,
        console_progress,
    );
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

#[cfg(test)]
mod tests {
    use super::{find_duplicate_files, ChecksumGenerator};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("backup-checker-{name}-{nanos}"));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn finds_duplicate_files_by_checksum() {
        let dir = create_test_dir("duplicates");
        let nested = dir.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(dir.join("a.txt"), "same content").unwrap();
        fs::write(nested.join("b.txt"), "same content").unwrap();
        fs::write(dir.join("unique.txt"), "different content").unwrap();

        let mut duplicates = find_duplicate_files(
            &dir.to_str().unwrap().to_string(),
            10,
            &ChecksumGenerator::CRC32,
            false,
        );
        duplicates.sort();

        assert_eq!(duplicates.len(), 1);
        assert_eq!(
            duplicates[0],
            vec![
                dir.join("a.txt").to_str().unwrap().to_string(),
                nested.join("b.txt").to_str().unwrap().to_string(),
            ]
        );

        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn ignores_unique_files() {
        let dir = create_test_dir("unique");
        fs::write(dir.join("a.txt"), "first").unwrap();
        fs::write(dir.join("b.txt"), "second").unwrap();

        let duplicates = find_duplicate_files(
            &dir.to_str().unwrap().to_string(),
            10,
            &ChecksumGenerator::CRC32,
            false,
        );

        assert!(duplicates.is_empty());

        fs::remove_dir_all(dir).unwrap();
    }
}
