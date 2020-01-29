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

use super::super::YamlBackedStoreError;

#[derive(Debug)]
pub enum DefaultStoreError {
    NotSet(String),
    OperationFailed(Box<dyn Error>),
}

impl Error for DefaultStoreError {}

impl fmt::Display for DefaultStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DefaultStoreError::NotSet(msg) => write!(f, "Default not set: {}", msg),
            DefaultStoreError::OperationFailed(err) => {
                write!(f, "The underlying store encountered an error {}", err)
            }
        }
    }
}

impl From<YamlBackedStoreError> for DefaultStoreError {
    fn from(err: YamlBackedStoreError) -> DefaultStoreError {
        DefaultStoreError::OperationFailed(Box::new(err))
    }
}
