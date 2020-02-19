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

use crate::admin::messages::BuilderError;

#[derive(Debug)]
pub enum RuleError {
    InvalidFormat(String),
    InternalError(String),
    SerdeError(serde_yaml::Error),
}

impl std::error::Error for RuleError {}

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            RuleError::InvalidFormat(ref s) => {
                write!(f, "The data provided is not in the correct format: {}", s)
            }
            RuleError::InternalError(ref s) => write!(
                f,
                "The rule encountered an internal error while processing data: {}",
                s
            ),
            RuleError::SerdeError(ref err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl From<serde_yaml::Error> for RuleError {
    fn from(err: serde_yaml::Error) -> Self {
        RuleError::SerdeError(err)
    }
}

#[derive(Debug)]
pub enum TemplateParserError {
    RuleNotFound(String),
    SerdeError(serde_yaml::Error),
    IoError(std::io::Error),
    RuleError(RuleError),
    ServiceBuilderError(BuilderError),
}

impl std::error::Error for TemplateParserError {}

impl std::fmt::Display for TemplateParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            TemplateParserError::RuleNotFound(ref s) => write!(f, "Rule not found: {}", s),
            TemplateParserError::SerdeError(ref err) => write!(f, "Deserialization error: {}", err),
            TemplateParserError::IoError(ref err) => {
                write!(f, "Template parser encountered and IO error: {}", err)
            }
            TemplateParserError::RuleError(ref err) => write!(f, "Failed to apply rule: {}", err),
            TemplateParserError::ServiceBuilderError(ref err) => {
                write!(f, "Failed to build service: {}", err)
            }
        }
    }
}

impl From<serde_yaml::Error> for TemplateParserError {
    fn from(err: serde_yaml::Error) -> Self {
        TemplateParserError::SerdeError(err)
    }
}

impl From<std::io::Error> for TemplateParserError {
    fn from(err: std::io::Error) -> Self {
        TemplateParserError::IoError(err)
    }
}

impl From<RuleError> for TemplateParserError {
    fn from(err: RuleError) -> Self {
        TemplateParserError::RuleError(err)
    }
}
