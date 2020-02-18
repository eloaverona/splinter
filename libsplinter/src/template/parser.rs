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
use std::fs::File;

use crate::admin::messages::{CreateCircuitBuilder, SplinterService, SplinterServiceBuilder};

use super::{rules::CircuitManagementTypeRule, Rule, RuleArgument, TemplateParserError};

#[derive(Deserialize, Debug)]
struct CircuitCreateTemplate {
    args: Vec<YamlRuleArgument>,
    rules: HashMap<String, serde_yaml::Value>,
}

#[derive(Deserialize, Debug)]
struct YamlRuleArgument {
    name: String,
    required: bool,
    #[serde(rename = "default")]
    default_value: Option<String>,
}

pub struct YamlCreateCircuitTemplateParser {
    circuit_rules: CreateCircuitRules,
    service_rules: ServiceDefinitionRules,
    rule_arguments: HashMap<String, Vec<RuleArgument>>,
}

impl Default for YamlCreateCircuitTemplateParser {
    fn default() -> Self {
        let mut rule_parser = YamlCreateCircuitTemplateParser {
            circuit_rules: CreateCircuitRules::default(),
            service_rules: ServiceDefinitionRules::default(),
            rule_arguments: HashMap::new(),
        };
        let rule = CircuitManagementTypeRule::default();
        rule_parser.register_circuit_rule(&rule.name(), Box::new(rule));
        rule_parser
    }
}

impl YamlCreateCircuitTemplateParser {
    pub fn parse_rules(
        mut self,
        path: &str,
        arg_values: HashMap<String, String>,
    ) -> Result<CreateCircuitBuilder, TemplateParserError> {
        let file = File::open(path)?;

        let template: CircuitCreateTemplate = serde_yaml::from_reader(file)?;

        for (rule_name, value) in template.rules.iter() {
            self.apply_rule(rule_name, &arg_values, &serde_yaml::to_vec(value)?)?
        }

        let mut builder = self.circuit_rules.builder();

        if self.service_rules.has_builders() {
            let services = self.service_rules.build_services()?;
            builder = builder.with_roster(&services);
        }

        Ok(builder)
    }

    pub fn register_service_rule(
        &mut self,
        name: &str,
        rule: Box<dyn Rule<Vec<SplinterServiceBuilder>>>,
    ) {
        let arguments = rule.get_arguments();
        self.rule_arguments.insert(name.to_string(), arguments);
        self.service_rules.register_rule(name, rule);
    }

    pub fn register_circuit_rule(&mut self, name: &str, rule: Box<dyn Rule<CreateCircuitBuilder>>) {
        let arguments = rule.get_arguments();
        self.rule_arguments.insert(name.to_string(), arguments);
        self.circuit_rules.register_rule(name, rule);
    }

    pub fn apply_rule(
        &mut self,
        name: &str,
        args: &HashMap<String, String>,
        values: &[u8],
    ) -> Result<(), TemplateParserError> {
        if self.circuit_rules.is_circuit_rule(name) {
            self.circuit_rules.apply_rule(name, args, values)?
        }

        if self.service_rules.is_service_rule(name) {
            self.service_rules.apply_rule(name, args, values)?
        }
        Ok(())
    }

    pub fn arguments(&self) -> HashMap<String, Vec<RuleArgument>> {
        self.rule_arguments.clone()
    }
}

struct CreateCircuitRules {
    rules: HashMap<String, Box<dyn Rule<CreateCircuitBuilder>>>,
    builder: CreateCircuitBuilder,
}

impl Default for CreateCircuitRules {
    fn default() -> Self {
        CreateCircuitRules {
            rules: HashMap::new(),
            builder: CreateCircuitBuilder::new(),
        }
    }
}

impl CreateCircuitRules {
    fn is_circuit_rule(&self, name: &str) -> bool {
        self.rules.get(name).is_some()
    }

    fn builder(self) -> CreateCircuitBuilder {
        self.builder
    }

    fn apply_rule(
        &mut self,
        name: &str,
        args: &HashMap<String, String>,
        values: &[u8],
    ) -> Result<(), TemplateParserError> {
        if let Some(rule) = self.rules.get_mut(name) {
            self.builder = rule.apply(values, args, self.builder.clone())?;
            return Ok(());
        }

        Err(TemplateParserError::RuleNotFound(format!(
            "Rule not found: {}",
            name
        )))
    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<CreateCircuitBuilder>>) {
        self.rules.insert(name.to_string(), rule);
    }
}

struct ServiceDefinitionRules {
    rules: HashMap<String, Box<dyn Rule<Vec<SplinterServiceBuilder>>>>,
    builders: Vec<SplinterServiceBuilder>,
}

impl Default for ServiceDefinitionRules {
    fn default() -> Self {
        ServiceDefinitionRules {
            rules: HashMap::new(),
            builders: vec![],
        }
    }
}

impl ServiceDefinitionRules {
    fn is_service_rule(&self, name: &str) -> bool {
        self.rules.get(name).is_some()
    }

    fn has_builders(&self) -> bool {
        !self.builders.is_empty()
    }

    fn apply_rule(
        &mut self,
        name: &str,
        args: &HashMap<String, String>,
        values: &[u8],
    ) -> Result<(), TemplateParserError> {
        if let Some(rule) = self.rules.get_mut(name) {
            self.builders = rule.apply(values, args, self.builders.clone())?;
            return Ok(());
        }
        Err(TemplateParserError::RuleNotFound(format!(
            "Rule not found: {}",
            name
        )))
    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterServiceBuilder>>>) {
        self.rules.insert(name.to_string(), rule);
    }

    fn build_services(self) -> Result<Vec<SplinterService>, TemplateParserError> {
        self.builders
            .into_iter()
            .try_fold(Vec::new(), |mut services, builder| {
                let service = builder
                    .build()
                    .map_err(TemplateParserError::ServiceBuilderError)?;
                services.push(service);
                Ok(services)
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{remove_file, File};
    use std::io::Write;
    use std::{env, panic, thread};

    static EXAMPLE_TEMPLATE_YAML: &[u8; 153] = br##"args:
    - name: admin-keys
      required: true
      default: $(a:SIGNER_PUB_KEY)
rules:
    set-management-type:
        management-type: "gameroom" "##;

    /*
     * Verifies YamlCreateCircuitTemplateParser correctly parses the template yaml file and
     * applies the set-management-type rule
     */
    #[test]
    fn test_parse_circuit_management_rule() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);
            let template_parser = YamlCreateCircuitTemplateParser::default();

            let result = template_parser.parse_rules(test_yaml_file_path, HashMap::new());
            assert!(result.is_ok());

            let circuit_create_builder = result.unwrap();
            assert_eq!(
                circuit_create_builder.circuit_management_type(),
                Some("gameroom".to_string())
            );
        })
    }

    fn run_test<T>(test: T) -> ()
    where
        T: FnOnce(&str) -> () + panic::UnwindSafe,
    {
        let test_yaml_file = temp_yaml_file_path();

        let test_path = test_yaml_file.clone();
        let result = panic::catch_unwind(move || test(&test_path));

        remove_file(test_yaml_file).unwrap();

        assert!(result.is_ok())
    }

    fn temp_yaml_file_path() -> String {
        let mut temp_dir = env::temp_dir();

        let thread_id = thread::current().id();
        temp_dir.push(format!("test_parse_template-{:?}.yaml", thread_id));
        temp_dir.to_str().unwrap().to_string()
    }

    fn write_yaml_file(file_path: &str) {
        println!("file_path {}", file_path);
        let mut file = File::create(file_path).expect("Error creating test template yaml file.");

        file.write_all(EXAMPLE_TEMPLATE_YAML)
            .expect("Error writing example template yaml.");
    }
}
