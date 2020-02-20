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

use std::collections::HashMap;

use super::{yaml_parser::v0_1, Error};

pub struct CircuitCreateTemplate {
    _version: String,
    _args: Vec<RuleArgument>,
    rules: Rules,
}

impl From<v0_1::YamlCircuitCreateTemplate> for CircuitCreateTemplate {
    fn from(create_circuit_templase: v0_1::YamlCircuitCreateTemplate) -> Self {
        CircuitCreateTemplate {
            _version: create_circuit_templase.version(),
            _args: create_circuit_templase
                .args()
                .into_iter()
                .map(RuleArgument::from)
                .collect(),
            rules: Rules::from(create_circuit_templase.rules()),
        }
    }
}

struct RuleArgument {
    _name: String,
    _required: bool,
    _default_value: Option<String>,
}

impl From<v0_1::YamlRuleArgument> for RuleArgument {
    fn from(arguments: v0_1::YamlRuleArgument) -> Self {
        RuleArgument {
            _name: arguments.name(),
            _required: arguments.required(),
            _default_value: arguments.default_value(),
        }
    }
}

struct Rules {
    set_management_type: CircuitManagement,
}

impl From<v0_1::YamlRules> for Rules {
    fn from(rules: v0_1::YamlRules) -> Self {
        Rules {
            set_management_type: CircuitManagement::from(rules.set_management_type()),
        }
    }
}

struct CircuitManagement {
    management_type: String,
}

impl From<v0_1::YamlCircuitManagement> for CircuitManagement {
    fn from(yaml_circuit_management: v0_1::YamlCircuitManagement) -> Self {
        CircuitManagement {
            management_type: yaml_circuit_management.management_type(),
        }
    }
}
