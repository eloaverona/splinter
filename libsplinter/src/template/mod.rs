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
mod parser;
mod rules;

use std::collections::HashMap;

pub use error::{RuleError, TemplateParserError};
pub use parser::YamlCreateCircuitTemplateParser;

pub trait Rule<T> {
    /// Rule name
    fn name(&self) -> String;

    /// Takes in serialized data, and based on the Rule, applies data to a builder
    fn apply(
        &mut self,
        values: &[u8],
        args: &HashMap<String, String>,
        builder: T,
    ) -> Result<T, RuleError>;

    /// Arguments the rule defines
    fn get_arguments(&self) -> Vec<RuleArgument>;
}

#[derive(Clone)]
/// Definition of an argument for Rule
pub struct RuleArgument {
    name: String,
    required: bool,
    default_value: Option<String>,
}

impl RuleArgument {
    pub fn new_required_argument(name: &str) -> RuleArgument {
        RuleArgument {
            name: name.to_string(),
            required: true,
            default_value: None,
        }
    }

    pub fn new_optional_argument(name: &str, default_value: &str) -> RuleArgument {
        RuleArgument {
            name: name.to_string(),
            required: false,
            default_value: Some(default_value.into()),
        }
    }
}
