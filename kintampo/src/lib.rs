extern crate edn;
#[macro_use]
extern crate log;
extern crate walkdir;

use std::path::Path;
use walkdir::{DirEntry, WalkDir};

fn is_directory(entry: &DirEntry) -> bool {
    entry.file_type().is_dir()
}

// TODO Re-research virtual fs for testing
pub fn all_dirs(root_dir: &std::path::PathBuf) -> Vec<String> {
    let walker = WalkDir::new(&root_dir).into_iter();
    let mut paths: Vec<String> = vec![];
    for entry in walker.filter_entry(|e| is_directory(e)) {
        let entry: walkdir::DirEntry = entry.unwrap();
        trace!("{}", entry.path().display());
        let path = entry.path().to_str().unwrap();
        paths.push(format!("\"{}\"",path));
    }
    paths
}

pub fn new_path_envelope(path_name: &str) -> String {
    let mut new_envelope = String::with_capacity(path_name.len() + 6);
    new_envelope.push_str("NEW://");
    new_envelope.push_str(&path_name);
    new_envelope
}

pub fn parse_envelope(envelope_name: &str) -> (String, &Path) {
    let colon_idx = envelope_name.find(':').unwrap();
    let path_start_idx = colon_idx + 3;
    let op_end_idx = colon_idx;
    (
        envelope_name.get(0..op_end_idx).unwrap().to_owned(),
        Path::new(envelope_name.get(path_start_idx..).unwrap())
    )
}

pub fn parse_edn_vector(s: &str) -> Vec<String> {
    match edn::parser::Parser::new(s).read().unwrap().expect("bad EDN") {
        edn::Value::Vector(items) => {
            items.into_iter().map(|i| match i { edn::Value::String(s) => s, _ => "".to_owned()}).collect()
        },
        _ => {
            error!("Only EDN vectors of strings supported.");
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_edn_vector() {
        assert_eq!(vec!["a".to_owned(), "b".to_owned()], parse_edn_vector("[\"a\",\"b\"]"));
    }

    #[test]
    fn test_parse_edn_vector_empty_as_error() {
        let expected: Vec<String> = vec![];
        assert_eq!(expected, parse_edn_vector(":something-else"));
    }

    #[test]
    fn test_new_path_envelope() {
        assert_eq!("NEW:///tmp/testing", new_path_envelope("/tmp/testing"));
    }

    #[test]
    fn test_path_from_envelope() {
        let (actual_op, actual_path) = parse_envelope("ADHOC:///tmp/foo");
        let (expected_op, expected_path) = ("ADHOC".to_owned(), Path::new("/tmp/foo"));
        assert_eq!(expected_op, actual_op);
        assert_eq!(expected_path, actual_path);
    }
}
