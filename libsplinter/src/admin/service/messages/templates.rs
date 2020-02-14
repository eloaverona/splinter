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

use super::builders::{CreateCircuitBuilder, SplinterNodeBuilder, SplinterServiceBuilder};
use std::collections::HashMap;

struct CreateCircuitRules {
    circuit_rules: HashMap<String, Box<dyn Rule<CreateCircuitBuilder>>>,
    service_rules: ServiceDefinitionRules,
    node_rules: NodeDefinitionRules,
    builder: CreateCircuitBuilder,
    args: HashMap<String, Vec<RuleArgument>>,
}

pub struct RuleArgument {
    key: String,
    required: bool,
    default_value: Option<String>,
}

impl RuleArgument {
    fn new_required_argument(key: &str) -> RuleArgument {
        RuleArgument {
            key: key.to_string(),
            required: true,
            default_value: None,
        }
    }

    fn new_optional_argument(key: &str, default_value: &str) -> RuleArgument {
        RuleArgument {
            key: key.to_string(),
            required: false,
            default_value: Some(default_value.into()),
        }
    }
}

struct ServiceDefinitionRules {
    rules: HashMap<String, Box<dyn Rule<Vec<SplinterServiceBuilder>>>>,
    builders: Vec<SplinterServiceBuilder>,
}

impl ServiceDefinitionRules {
    fn apply_rule(&mut self, name: &str, values: &[u8]) {
        if let Some(rule) = self.rules.get_mut(name) {
            self.builders = rule.apply(values, self.builders.clone());
            return ();
        }
    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterServiceBuilder>>>) {
        //let arguments = rule.get_arguments();
        //    self.args.insert(name.to_string(), arguments);
        self.rules.insert(name.to_string(), rule);
    }
}

struct NodeDefinitionRules {
    rules: HashMap<String, Box<dyn Rule<Vec<SplinterNodeBuilder>>>>,
    builders: Vec<SplinterNodeBuilder>,
}

impl NodeDefinitionRules {
    fn apply_rule(&mut self, name: &str, values: &[u8]) {
        if let Some(rule) = self.rules.get_mut(name) {
            self.builders = rule.apply(values, self.builders.clone());
            return ();
        }
    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterNodeBuilder>>>) {
        //let arguments = rule.get_arguments();
        //    self.args.insert(name.to_string(), arguments);
        self.rules.insert(name.to_string(), rule);
    }
}

impl CreateCircuitRules {
    fn apply_rule(&mut self, name: &str, values: &[u8]) {
        if let Some(rule) = self.circuit_rules.get_mut(name) {
            self.builder = rule.apply(values, self.builder.clone());
            return ();
        }

        self.service_rules.apply_rule(name, values);

        self.node_rules.apply_rule(name, values);
        // if let Some(rule) = self.node_rules.get(name) {
        //     rule.apply(values);
        //     return ();
        // }
    }

    fn register_service_rule(
        &mut self,
        name: &str,
        rule: Box<dyn Rule<Vec<SplinterServiceBuilder>>>,
    ) {
        let arguments = rule.get_arguments();
        self.args.insert(name.to_string(), arguments);
        self.service_rules.register_rule(name, rule);
    }

    fn register_circuit_rule(&mut self, name: &str, rule: Box<dyn Rule<CreateCircuitBuilder>>) {
        let arguments = rule.get_arguments();
        self.args.insert(name.to_string(), arguments);
        self.circuit_rules.insert(name.to_string(), rule);
    }

    fn register_node_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterNodeBuilder>>>) {
        let arguments = rule.get_arguments();
        self.args.insert(name.to_string(), arguments);
        self.node_rules.register_rule(name, rule);
    }
}

pub enum RuleType {
    Circuit(CreateCircuitBuilder),
    Service(Vec<SplinterServiceBuilder>),
    Node(Vec<SplinterNodeBuilder>),
}

pub trait Rule<T> {
    fn name(&self) -> String;
    fn apply(&mut self, values: &[u8], builder: T) -> T;
    fn get_arguments(&self) -> Vec<RuleArgument>;
}

// pub struct Rule<T> {
//     name: String,
//     fields: T
// }
