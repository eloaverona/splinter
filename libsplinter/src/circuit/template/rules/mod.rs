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

mod set_management_type;

use std::convert::TryFrom;

use super::{yaml_parser::v1, Builders, CircuitTemplateError};
use set_management_type::CircuitManagement;

pub struct Rules {
    set_management_type: Option<CircuitManagement>,
}

impl Rules {
    pub fn apply_rules(
        &self,
        builders: &mut Builders,
        template_arguments: &[RuleArgument],
    ) -> Result<(), CircuitTemplateError> {
        let mut circuit_builder = builders.create_circuit_builder();

        if let Some(circuit_management) = &self.set_management_type {
            circuit_builder = circuit_management.apply_rule(circuit_builder)?;
        }
        builders.set_create_circuit_builder(circuit_builder);
        Ok(())
    }
}

impl From<v1::Rules> for Rules {
    fn from(rules: v1::Rules) -> Self {
        Rules {
            set_management_type: rules
                .set_management_type()
                .map(|val| CircuitManagement::from(val.clone())),
        }
    }
}

#[derive(Clone)]
pub struct RuleArgument {
    name: String,
    required: bool,
    default_value: Option<String>,
    user_value: Option<String>,
}

impl RuleArgument {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn required(&self) -> bool {
        self.required
    }

    pub fn default_value(&self) -> Option<&String> {
        self.default_value.as_ref()
    }

    pub fn user_value(&self) -> Option<&String> {
        self.user_value.as_ref()
    }

    pub fn set_user_value(&mut self, value: &str) {
        self.user_value = Some(value.to_string())
    }
}

impl TryFrom<v1::RuleArgument> for RuleArgument {
    type Error = CircuitTemplateError;
    fn try_from(arguments: v1::RuleArgument) -> Result<Self, Self::Error> {
        Ok(RuleArgument {
            name: strip_arg_marker(arguments.name())?,
            required: arguments.required(),
            default_value: arguments.default_value().map(String::from),
            user_value: None,
        })
    }
}

fn strip_arg_marker(key: &str) -> Result<String, CircuitTemplateError> {
    if key.starts_with("$(") && key.ends_with(')') {
        let mut key = key.to_string();
        key.pop();
        Ok(key
            .get(2..)
            .ok_or_else(|| {
                CircuitTemplateError::new(&format!("{} is not a valid argument name", key))
            })?
            .to_string()
            .to_lowercase())
    } else {
        Err(CircuitTemplateError::new(&format!(
            "{} is not a valid argument name",
            key
        )))
    }
}
