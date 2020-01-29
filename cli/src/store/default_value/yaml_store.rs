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

use serde::{Deserialize, Serialize};

use super::super::yaml_store::YamlBackedStore;
use super::{DefaultStoreError, DefaultValue, DefaultValueStore};

const DEFAULT_FILE_NAME: &str = "circuit_defaults";

#[derive(Serialize, Deserialize)]
pub struct SerdeDefaultValue {
    key: String,
    value: String,
}

impl Into<DefaultValue> for SerdeDefaultValue {
    fn into(self) -> DefaultValue {
        DefaultValue {
            key: self.key,
            value: self.value,
        }
    }
}

impl From<&DefaultValue> for SerdeDefaultValue {
    fn from(default_value: &DefaultValue) -> SerdeDefaultValue {
        SerdeDefaultValue {
            key: default_value.key.clone(),
            value: default_value.value.clone(),
        }
    }
}

pub struct FileBackedDefaultStore {
    file_name: String,
}

impl Default for FileBackedDefaultStore {
    fn default() -> Self {
        FileBackedDefaultStore {
            file_name: DEFAULT_FILE_NAME.to_owned(),
        }
    }
}

impl YamlBackedStore<SerdeDefaultValue> for FileBackedDefaultStore {}

impl DefaultValueStore for FileBackedDefaultStore {
    fn set_default_value(&self, new_default_value: &DefaultValue) -> Result<(), DefaultStoreError> {
        let mut defaults = Self::read_data_from_file(&self.file_name)?;
        let existing_default_index = defaults.iter().enumerate().find_map(|(index, default)| {
            if default.key == new_default_value.key() {
                Some(index)
            } else {
                None
            }
        });
        if let Some(index) = existing_default_index {
            defaults.remove(index);
        }

        defaults.push(SerdeDefaultValue::from(new_default_value));

        Self::write_to_file(&self.file_name, &defaults)?;

        Ok(())
    }
    fn unset_default_value(&self, default_key: &str) -> Result<(), DefaultStoreError> {
        let mut all_defaults = Self::read_data_from_file(&self.file_name)?;

        let key_index = all_defaults
            .iter()
            .enumerate()
            .find_map(|(index, default)| {
                if default.key == default_key {
                    Some(index)
                } else {
                    None
                }
            });

        if let Some(index) = key_index {
            all_defaults.remove(index);
            Self::write_to_file(&self.file_name, &all_defaults)?;
        } else {
            return Err(DefaultStoreError::NotSet(format!(
                "Default value for {} not found",
                default_key
            )));
        }

        Ok(())
    }

    fn list_default_values(&self) -> Result<Vec<DefaultValue>, DefaultStoreError> {
        let serde_defaults = Self::read_data_from_file(&self.file_name)?;
        let defaults = serde_defaults
            .into_iter()
            .map(|default_value| default_value.into())
            .collect::<Vec<DefaultValue>>();
        Ok(defaults)
    }

    fn get_default_value(&self, key: &str) -> Result<Option<DefaultValue>, DefaultStoreError> {
        let defaults = Self::read_data_from_file(&self.file_name)?;

        let default_value = defaults.into_iter().find_map(|default_value| {
            if default_value.key == key {
                return Some(default_value.into());
            }
            None
        });

        Ok(default_value)
    }
}
