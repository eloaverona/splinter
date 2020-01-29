// Copyright 2018-2020 Cargill Incorporated
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

use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

use super::YamlBackedStoreError;

const DEFAULTS_PATH: &str = ".splinter";

pub(super) trait YamlBackedStore<T: Serialize + DeserializeOwned> {
    fn open_write_file(file_name: &str) -> Result<File, YamlBackedStoreError> {
        let mut file_path = Self::get_file_path()?;
        file_path.push(file_name);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)?;

        Ok(file)
    }

    fn open_read_file(file_name: &str) -> Result<File, YamlBackedStoreError> {
        let mut file_path = Self::get_file_path()?;
        file_path.push(file_name);
        Self::create_file(&file_path)?;
        let file = OpenOptions::new().read(true).open(&file_path)?;

        Ok(file)
    }

    fn create_file(file_path: &PathBuf) -> Result<(), YamlBackedStoreError> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)?;
        Ok(())
    }

    fn write_to_file(file_name: &str, data: &[T]) -> Result<(), YamlBackedStoreError> {
        let file = Self::open_write_file(file_name)?;
        serde_yaml::to_writer(file, &data)?;
        Ok(())
    }

    fn read_data_from_file(file_name: &str) -> Result<Vec<T>, YamlBackedStoreError> {
        let file = Self::open_read_file(file_name)?;
        if file.metadata()?.len() == 0 {
            return Ok(vec![]);
        }
        let data: Vec<T> = serde_yaml::from_reader(file)?;
        Ok(data)
    }

    fn get_file_path() -> Result<PathBuf, YamlBackedStoreError> {
        let mut path = dirs::home_dir().unwrap_or_default();
        path.push(DEFAULTS_PATH);
        fs::create_dir_all(&path)?;
        Ok(path)
    }
}
