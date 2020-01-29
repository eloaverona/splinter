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

use std::error::Error;
use std::fmt;

use crate::store::default_value::DefaultStoreError;

#[derive(Debug)]
pub enum DefaultValueManagerError {
    StoreError(DefaultStoreError),
    NotSet(String),
    AlreadySet(String),
}

impl Error for DefaultValueManagerError {}

impl fmt::Display for DefaultValueManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DefaultValueManagerError::StoreError(err) => {
                write!(f, "The store returned an error: {}", err)
            }
            DefaultValueManagerError::NotSet(err) => write!(f, "Default value is not set: {}", err),
            DefaultValueManagerError::AlreadySet(err) => {
                write!(f, "Default value is already set: {}", err)
            }
        }
    }
}

impl From<DefaultStoreError> for DefaultValueManagerError {
    fn from(err: DefaultStoreError) -> DefaultValueManagerError {
        DefaultValueManagerError::StoreError(err)
    }
}
