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

mod error;

use crate::store::default_value::{DefaultValueStore, FileBackedDefaultStore};

pub use crate::store::default_value::DefaultValue;
pub use error::DefaultValueManagerError;

const MANAGEMENT_TYPE_KEY: &str = "management_type";
const SERVICE_TYPE_KEY: &str = "service_type";

pub struct DefaultValueManager {
    store: Box<dyn DefaultValueStore>,
}

impl Default for DefaultValueManager {
    fn default() -> Self {
        DefaultValueManager {
            store: Box::new(FileBackedDefaultStore::default()),
        }
    }
}

impl DefaultValueManager {
    pub fn set_default_service_type(
        &self,
        value: &str,
        overwrite: bool,
    ) -> Result<(), DefaultValueManagerError> {
        self.set_default(SERVICE_TYPE_KEY, value, overwrite)?;
        Ok(())
    }

    pub fn unset_default_service_type(&self) -> Result<(), DefaultValueManagerError> {
        self.unset_default(SERVICE_TYPE_KEY)?;
        Ok(())
    }

    pub fn get_default_service_type(&self) -> Result<DefaultValue, DefaultValueManagerError> {
        self.get_default(SERVICE_TYPE_KEY)
    }

    pub fn set_default_management_type(
        &self,
        value: &str,
        overwrite: bool,
    ) -> Result<(), DefaultValueManagerError> {
        self.set_default(MANAGEMENT_TYPE_KEY, value, overwrite)?;
        Ok(())
    }

    pub fn unset_default_management_type(&self) -> Result<(), DefaultValueManagerError> {
        self.unset_default(MANAGEMENT_TYPE_KEY)?;
        Ok(())
    }

    pub fn get_default_management_type(&self) -> Result<DefaultValue, DefaultValueManagerError> {
        self.get_default(MANAGEMENT_TYPE_KEY)
    }

    pub fn list_defaults(&self) -> Result<Vec<DefaultValue>, DefaultValueManagerError> {
        let defaults = self.store.list_default_values()?;
        Ok(defaults)
    }

    fn set_default(
        &self,
        key: &str,
        value: &str,
        overwrite: bool,
    ) -> Result<(), DefaultValueManagerError> {
        if !overwrite && self.store.get_default_value(key)?.is_some() {
            return Err(DefaultValueManagerError::AlreadySet(format!(
                "Default value for {} is already in use",
                key
            )));
        }

        let default_value = DefaultValue::new(key, value);
        self.store.set_default_value(&default_value)?;
        Ok(())
    }

    fn unset_default(&self, key: &str) -> Result<(), DefaultValueManagerError> {
        self.store.unset_default_value(key)?;
        Ok(())
    }

    fn get_default(&self, key: &str) -> Result<DefaultValue, DefaultValueManagerError> {
        match self.store.get_default_value(key)? {
            Some(value) => Ok(value),
            None => Err(DefaultValueManagerError::NotSet(format!(
                "Default value for {} is not set",
                key
            ))),
        }
    }
}
