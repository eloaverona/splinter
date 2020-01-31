// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Result as IoResult;
use std::io::{BufRead, BufReader, Write};
use std::io::{Error as IoError, ErrorKind};
use std::path::PathBuf;

const DEFAULTS_PATH: &str = ".splinter";

pub fn open_or_create_file(file_name: &str) -> IoResult<File> {
    let mut file_path = get_file_path();
    fs::create_dir_all(&file_path)?;

    file_path.push(file_name);

    OpenOptions::new()
        .read(true)
        .append(true)
        .write(true)
        .create(true)
        .open(&file_path)
}

pub fn overwrite_file(file_name: &str, data: HashMap<String, String>) -> IoResult<()> {
    let mut file_path = get_file_path();
    fs::create_dir_all(&file_path)?;

    file_path.push(file_name);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&file_path)?;

    data.iter()
        .try_for_each(|(key, value)| writeln!(&mut file, "{} {}", key, value))?;
    Ok(())
}

pub fn write_key_value_to_file(file_name: &str, key: &str, value: &str) -> IoResult<()> {
    let mut file = open_or_create_file(file_name)?;
    writeln!(&mut file, "{} {}", key, value)?;
    Ok(())
}

pub fn fetch_key_from_file(file_name: &str, key: &str) -> IoResult<Option<String>> {
    let file = open_or_create_file(file_name)?;
    let buf_reader = BufReader::new(file);
    let mut all_data = buf_reader.lines();
    let endpoint = all_data.find_map(|line| {
        let ln = match line {
            Ok(str_line) => str_line,
            Err(err) => {
                debug!("Unable to read line from alias file: {}", err);
                return None;
            }
        };
        let mut it = ln.split_whitespace();
        let alias = it.next().unwrap_or_default().to_string();
        if alias == key {
            let endpoint = it.next().unwrap_or_default().to_string();
            return Some(endpoint);
        }
        None
    });
    Ok(endpoint)
}

pub fn list_keys_from_file(file_name: &str) -> IoResult<HashMap<String, String>> {
    let file = open_or_create_file(file_name)?;
    let buf_reader = BufReader::new(file);
    let mut lines = buf_reader.lines();
    lines.try_fold(HashMap::new(), |mut acc, line| {
        let raw_line = line?;
        let mut it = raw_line.split_whitespace();
        let alias = it
            .next()
            .ok_or_else(|| IoError::new(ErrorKind::Other, "File is not formatted correctly"))?
            .to_string();

        let endpoint = it
            .next()
            .ok_or_else(|| IoError::new(ErrorKind::Other, "File is not formatted correctly"))?
            .to_string();

        acc.insert(alias, endpoint);

        Ok(acc)
    })
}

pub fn get_file_path() -> std::path::PathBuf {
    let mut path = dirs::home_dir().unwrap_or_default();
    path.push(DEFAULTS_PATH);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{remove_file, File};
    use std::panic;
    use std::thread;

    /// Asserts that given correct input write_key_value_to_file writes the data
    /// to the file correctly.
    #[test]
    fn test_write_data_ok() {
        run_test(|file_name| {
            assert!(write_key_value_to_file(&file_name, "node-123", "localhost:8080").is_ok());

            let file = File::open(&get_full_file_path(&file_name)).expect("Error opening file.");
            let mut buf_reader = BufReader::new(file);
            let mut line_1 = String::new();
            buf_reader
                .read_line(&mut line_1)
                .expect("Failed to read line");
            assert_eq!(line_1, "node-123 localhost:8080\n");
        })
    }

    /// Asserts that given correct input write_key_value_to_file writes the node appends new node
    /// information at the end of the file and does not overwrite it
    #[test]
    fn test_write_data_append_ok() {
        run_test(|file_name| {
            assert!(write_key_value_to_file(&file_name, "node-123", "localhost:8080").is_ok());

            let full_file_path = get_full_file_path(&file_name);

            let file = File::open(&full_file_path).expect("Error opening file.");
            let mut buf_reader = BufReader::new(file);
            let mut line_1 = String::new();
            buf_reader
                .read_line(&mut line_1)
                .expect("Failed to read line");
            assert_eq!(line_1, "node-123 localhost:8080\n");

            assert!(write_key_value_to_file(&file_name, "node-456", "localhost:8081").is_ok());

            let file = File::open(&full_file_path).expect("Error opening file.");

            let mut buf_reader = BufReader::new(file);
            let mut line_1 = String::new();
            buf_reader
                .read_line(&mut line_1)
                .expect("Failed to read line");
            assert_eq!(line_1, "node-123 localhost:8080\n".to_string());

            let mut line_2 = String::new();
            buf_reader
                .read_line(&mut line_2)
                .expect("Failed to read line");
            assert_eq!(line_2, "node-456 localhost:8081\n".to_string());
        })
    }

    /// Asserts that given correct input fetch_key_from_file reads the value
    /// from the file correctly.
    #[test]
    fn test_fetch_key_from_file_ok() {
        run_test(|file_name| {
            let full_file_path = get_full_file_path(&file_name);

            write_to_file(
                "node-123 localhost:8080\nnode-456 localhost:8081",
                &full_file_path,
            );

            // Tests it works when the key is in the file
            let result = fetch_key_from_file(&file_name, "node-456");
            assert!(result.is_ok());
            let endpoint = result.unwrap();
            assert_eq!(endpoint, Some("localhost:8081".to_string()));

            // Tests it works when the key is not in the file
            let result = fetch_key_from_file(&file_name, "key-not-in-file");
            assert!(result.is_ok());
            let endpoint = result.unwrap();
            assert_eq!(endpoint, None);
        })
    }

    // Asserts that given correct input list_values_from_file reads the data
    // to the file correctly.
    #[test]
    fn test_list_keys_from_file_ok() {
        run_test(|file_name| {
            let full_file_path = get_full_file_path(&file_name);

            write_to_file(
                "node-123 localhost:8080\nnode-456 localhost:8081",
                &full_file_path,
            );

            // Tests it works when the node id is in the file
            let result = list_keys_from_file(&file_name);
            assert!(result.is_ok());
            let data = result.unwrap();
            assert_eq!(data.len(), 2);
            assert_eq!(
                data.get("node-123").map(String::from),
                Some("localhost:8080".to_string())
            );
            assert_eq!(
                data.get("node-456").map(String::from),
                Some("localhost:8081".to_string())
            );
        })
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce(String) -> () + panic::UnwindSafe,
    {
        let thread_id = thread::current().id();
        let file_name = format!("test_file-{:?}", thread_id);

        let full_file_path = get_full_file_path(&file_name);

        let result = panic::catch_unwind(move || test(file_name));

        remove_file(full_file_path).unwrap();

        assert!(result.is_ok())
    }

    fn get_full_file_path(file_name: &str) -> PathBuf {
        let mut full_file_path = get_file_path();
        full_file_path.push(file_name);
        full_file_path
    }

    fn write_to_file(data: &str, file_path: &PathBuf) {
        let mut file = File::create(file_path).expect("Error creating test file.");
        writeln!(&mut file, "{}", data).expect("Failed to write file");
    }
}
