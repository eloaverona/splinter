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

use serde_yaml::Error as SerdeError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum YamlBackedStoreError {
    IoError(IoError),
    SerdeError(SerdeError),
}

impl Error for YamlBackedStoreError {}

impl fmt::Display for YamlBackedStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            YamlBackedStoreError::IoError(err) => {
                write!(f, "The store encountered an IO error: {}", err)
            }
            YamlBackedStoreError::SerdeError(err) => write!(
                f,
                "The store encountered and serialization/deserialization error  {}",
                err
            ),
        }
    }
}

impl From<IoError> for YamlBackedStoreError {
    fn from(err: IoError) -> YamlBackedStoreError {
        YamlBackedStoreError::IoError(err)
    }
}

impl From<SerdeError> for YamlBackedStoreError {
    fn from(err: SerdeError) -> YamlBackedStoreError {
        YamlBackedStoreError::SerdeError(err)
    }
}
