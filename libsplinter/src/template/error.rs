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

#[derive(Debug)]
pub enum RuleError {
    InvalidFormat(String),
    SerdeError(serde_yaml::Error),
}

impl std::error::Error for RuleError {}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            RuleError::InvalidFormat(ref s) => {
                write!(f, "The data provided is not in the correct format: {}", s)
            }
            RuleError::SerdeError(ref err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl From<serde_yaml::Error> for RuleError {
    fn from(err: serde_yaml::Error) -> Self {
        RuleError::SerdeError(err)
    }
}
