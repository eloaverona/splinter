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
use std::fs::File;
use serde_yaml;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
struct CircuitCreateTemplate {
    args: Vec<RuleArgument>,
    rules: HashMap<String, serde_yaml::Value>
}

#[derive(Debug)]
struct CircuitManagementTypeRule {
    name: String,
}

impl Default for CircuitManagementTypeRule {
    fn default() -> Self {
        CircuitManagementTypeRule {
            name: "set-management-type".to_string()
        }
    }
}

impl Rule<CreateCircuitBuilder> for CircuitManagementTypeRule {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn apply(&mut self, values: &[u8], _: &HashMap<String, String>, builder: CreateCircuitBuilder) -> CreateCircuitBuilder {
        let management_type = serde_yaml::from_slice::<HashMap<String, String>>(values).expect("failed to parse managment type");
        let circuit_management_type = management_type.get("management-type").unwrap();
        builder.with_circuit_management_type(circuit_management_type)
    }

    fn get_arguments(&self) -> Vec<RuleArgument> {
        vec![]
    }
}

#[derive(Debug)]
struct CreateServicesRule {
    name: String,
    args: Vec<RuleArgument>
}

#[derive(Deserialize)]
struct CreateServices {
    service_type: String,
    service_args: Vec<HashMap<String, String>>,
    first_service: String
}

impl Default for CreateServicesRule {
    fn default() -> Self {
        let nodes_arg = RuleArgument{
            name: "NODES".to_string(),
            required: true,
            default_value: None,
        };
        CreateServicesRule {
            name: "create-services".to_string(),
            args: vec![nodes_arg]
        }
    }
}

impl Rule<Vec<SplinterServiceBuilder>> for CreateServicesRule {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn apply(&mut self, values: &[u8], args: &HashMap<String, String>, _: Vec<SplinterServiceBuilder>) -> Vec<SplinterServiceBuilder> {
        let create_services = serde_yaml::from_slice::<CreateServices>(values).expect("failed to parse managment type");
        let nodes = args.get("NODES").expect("No nodes").split(",").collect::<Vec<String>>();
        let valid_chars = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let first_service = create_services.first_service
        for node in nodes {
            let splinter_service_builder = SplinterServiceBuilder::new().with_service_id(service_id: &str)
        }

        vec![]
        // let circuit_management_type = management_type.get("management-type").unwrap();
        // builder.with_circuit_management_type(circuit_management_type)c
    }

    fn get_arguments(&self) -> Vec<RuleArgument> {
        self.args.clone()
    }
}

fn get_next_service_id(current_id: &str) -> String {
    let last_char = current_id.to_string().pop().expect("charr");
    if last_char.is_numeric()
}


struct YamlCreateCircuitTemplateParser {
    circuit_rules: CreateCircuitRules,
    service_rules: ServiceDefinitionRules,
    node_rules: NodeDefinitionRules,
    //builder: CreateCircuitBuilder,
    args: HashMap<String, Vec<RuleArgument>>,

}

impl Default for YamlCreateCircuitTemplateParser {
    fn default() -> Self {
        let mut rule_parser = YamlCreateCircuitTemplateParser {
            circuit_rules: CreateCircuitRules::default(),
            service_rules: ServiceDefinitionRules::default(),
            node_rules: NodeDefinitionRules::default(),
            args: HashMap::new()
        };
        let rule  = CircuitManagementTypeRule::default();
        rule_parser.register_circuit_rule(&rule.name(), Box::new(rule));
        rule_parser
    }
}

impl YamlCreateCircuitTemplateParser {
    fn parse_rules(mut self, path: &str, arg_values: HashMap<String, String>) -> CreateCircuitBuilder {
        let file = File::open(path).expect("err");
        let template: CircuitCreateTemplate = serde_yaml::from_reader(file).expect("err open file");
        println!("template {:?}", template);
        //let rules = template.get("rules").unwrap();
        //
        // let args = template.get("args").unwrap()

        //for rule in template.rules.iter() {
        for (rule_name, value) in template.rules.iter() {
            self.apply_rule(rule_name, &arg_values, &serde_yaml::to_vec(value).expect("errr"))

        }
        //}

        CreateCircuitBuilder::new()
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
        self.circuit_rules.register_rule(name, rule);
    }

    fn register_node_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterNodeBuilder>>>) {
        let arguments = rule.get_arguments();
        self.args.insert(name.to_string(), arguments);
        self.node_rules.register_rule(name, rule);
    }

    fn apply_rule(&mut self, name: &str, args: &HashMap<String, String>, values: &[u8]) {
        // if let Some(rule) = self.circuit_rules.get_mut(name) {
        //     self.builder = rule.apply(values, self.builder.clone());
        //     return ();
        // }
        println!("got to template parser apply_rule");
        self.circuit_rules.apply_rule(name, args, values); //TO DO need way to check which kind of rule
        //
        // self.service_rules.apply_rule(name, values);
        //
        // self.node_rules.apply_rule(name, values);
        // if let Some(rule) = self.node_rules.get(name) {
        //     rule.apply(values);
        //     return ();
        // }
    }
}

struct CreateCircuitRules {
    rules: HashMap<String, Box<dyn Rule<CreateCircuitBuilder>>>,
    // service_rules: ServiceDefinitionRules,
    // node_rules: NodeDefinitionRules,
    builder: CreateCircuitBuilder,
    //args: HashMap<String, Vec<RuleArgument>>,
}

impl Default for CreateCircuitRules {
    fn default() -> Self {
        CreateCircuitRules {
            rules: HashMap::new(),
            builder: CreateCircuitBuilder::new()
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RuleArgument {
    name: String,
    required: bool,
    #[serde(rename = "default")]
    default_value: Option<String>,
}

impl RuleArgument {
    fn new_required_argument(name: &str) -> RuleArgument {
        RuleArgument {
            name: name.to_string(),
            required: true,
            default_value: None,
        }
    }

    fn new_optional_argument(name: &str, default_value: &str) -> RuleArgument {
        RuleArgument {
            name: name.to_string(),
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
    fn apply_rule(&mut self, name: &str, args: &HashMap<String, String>, values: &[u8]) {
        if let Some(rule) = self.rules.get_mut(name) {
            self.builders = rule.apply(values, args, self.builders.clone());
            return ();
        }
    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterServiceBuilder>>>) {
        //let arguments = rule.get_arguments();
        //    self.args.insert(name.to_string(), arguments);
        self.rules.insert(name.to_string(), rule);
    }
}


impl Default for ServiceDefinitionRules {
    fn default() -> Self {
        ServiceDefinitionRules {
            rules: HashMap::new(),
            builders: vec![]
        }
    }
}

struct NodeDefinitionRules {
    rules: HashMap<String, Box<dyn Rule<Vec<SplinterNodeBuilder>>>>,
    builders: Vec<SplinterNodeBuilder>,
}

impl NodeDefinitionRules {
    fn apply_rule(&mut self, name: &str, args: &HashMap<String, String>, values: &[u8]) {
        if let Some(rule) = self.rules.get_mut(name) {
            self.builders = rule.apply(values, args, self.builders.clone());
            return ();
        }
    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<Vec<SplinterNodeBuilder>>>) {
        //let arguments = rule.get_arguments();
        //    self.args.insert(name.to_string(), arguments);
        self.rules.insert(name.to_string(), rule);
    }
}

impl Default for NodeDefinitionRules {
    fn default() -> Self {
        NodeDefinitionRules {
            rules: HashMap::new(),
            builders: vec![]
        }
    }
}

impl CreateCircuitRules {
    fn apply_rule(&mut self, name: &str, args: &HashMap<String, String>, values: &[u8]) {
        println!("got to CreateCircuitRules apply_rule");
        println!("circuit_create_rule {:?}", self.rules);
        println!("name {}", name);


        if let Some(rule) = self.rules.get_mut(name) {
            println!("found rule");

            self.builder = rule.apply(values, args, self.builder.clone());
            println!("builder {:?}", self.builder);
            return ();
        }

        println!("did not found rule");

    }

    fn register_rule(&mut self, name: &str, rule: Box<dyn Rule<CreateCircuitBuilder>>) {
        //let arguments = rule.get_arguments();
        //    self.args.insert(name.to_string(), arguments);
        self.rules.insert(name.to_string(), rule);
    }
    // fn apply_rule(&mut self, name: &str, values: &[u8]) {
    //     if let Some(rule) = self.circuit_rules.get_mut(name) {
    //         self.builder = rule.apply(values, self.builder.clone());
    //         return ();
    //     }
    //
    //     self.service_rules.apply_rule(name, values);
    //
    //     self.node_rules.apply_rule(name, values);
    //     // if let Some(rule) = self.node_rules.get(name) {
    //     //     rule.apply(values);
    //     //     return ();
    //     // }
    // }
}

pub enum RuleType {
    Circuit(CreateCircuitBuilder),
    Service(Vec<SplinterServiceBuilder>),
    Node(Vec<SplinterNodeBuilder>),
}

pub trait Rule<T: Debug> : Debug {
    fn name(&self) -> String;
    fn apply(&mut self, values: &[u8], args: &HashMap<String, String>, builder: T) -> T;
    fn get_arguments(&self) -> Vec<RuleArgument>;
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{remove_file, File};
    use std::io::Write;
    use std::{env, panic, thread};


    static EXAMPLE_RULES_YAML: &[u8; 172] = br##"args:
    - name: admin-keys
      required: true
      type: string
      default: $(a:SIGNER_PUB_KEY)
rules:
    set-management-type:
        management-type: "gameroom" "##;

    /*
     * Verifies parse_product_yaml returns valids ProductPayload with ProductCreateAction set from a yaml
     * containing a multiple Product definitions
     */
    #[test]
    fn test_parse_rule() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);
            let template_parser = YamlCreateCircuitTemplateParser::default();

            template_parser.parse_rules(test_yaml_file_path, HashMap::new());
            assert!(false)

            // let payload = parse_product_yaml(
            //     test_yaml_file_path,
            //     Action::ProductCreate(ProductCreateAction::default()),
            // )
            // .expect("Error parsing yaml");
            //
            // assert_eq!(make_create_product_payload(), payload[0]);
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
        let mut file = File::create(file_path).expect("Error creating test rules yaml file.");

        file.write_all(EXAMPLE_RULES_YAML)
            .expect("Error writting example product yaml.");
    }
  }
