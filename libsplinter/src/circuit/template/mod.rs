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
mod v0_1;
mod yaml_parser;

use std::collections::HashMap;

pub use error::Error;

use yaml_parser::{load_template, Template};

pub(self) use crate::admin::messages::{CreateCircuitBuilder, SplinterServiceBuilder};

pub struct Builders {
    create_circuit_builder: CreateCircuitBuilder,
    service_builders: Vec<SplinterServiceBuilder>,
}

impl Builders {
    pub fn try_from_template(
        path: &str,
        arguments: &HashMap<String, String>,
    ) -> Result<Builders, Error> {
        let template = load_template(path)?;
        let mut builders = Builders {
            create_circuit_builder: CreateCircuitBuilder::new(),
            service_builders: vec![],
        };
        match template {
            Template::V0_1(template) => {
                let native_template = v0_1::CircuitCreateTemplate::from(template);
                native_template.apply_rules(&mut builders, &arguments)?;
            }
        }
        Ok(builders)
    }

    pub fn set_create_circuit_builder(&mut self, builder: CreateCircuitBuilder) {
        self.create_circuit_builder = builder;
    }

    pub fn set_service_builders(&mut self, builders: Vec<SplinterServiceBuilder>) {
        self.service_builders = builders;
    }

    pub fn create_circuit_builder(&self) -> CreateCircuitBuilder {
        self.create_circuit_builder.clone()
    }

    pub fn service_builders(&self) -> Vec<SplinterServiceBuilder> {
        self.service_builders.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{remove_file, File};
    use std::io::Write;
    use std::{env, panic, thread};

    static EXAMPLE_TEMPLATE_YAML: &[u8; 407] = br##"version: v0.1
args:
    - name: admin-keys
      required: false
      default: $(a:SIGNER_PUB_KEY)
rules:
    set-management-type:
        management-type: "gameroom"
    create-services:
        service-type: 'scabbard'
        service-args:
        - key: 'admin-keys'
          value: $(admin-keys)
        - key: 'peer-services'
          value: '$(r:ALL_OTHER_SERVICES)'
        first-service: 'a000' "##;

    /*
     * Verifies that Builders can be parsed from template v0.1 and correctly
     * applies the set-management-type rule
     */
    #[test]
    fn test_builds_template_v0_1() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);
            let mut args = HashMap::new();
            args.insert(
                "NODES".to_string(),
                "alpha-node-000,beta-node-000".to_string(),
            );
            let builders = Builders::try_from_template(test_yaml_file_path, &args)
                .expect("Error getting builders from templates");
            let circuit_create_builder = builders.create_circuit_builder();
            assert_eq!(
                circuit_create_builder.circuit_management_type(),
                Some("gameroom".to_string())
            );
            let service_builders = builders.service_builders();
            let service_alpha_node = service_builders
                .iter()
                .find(|service| service.allowed_nodes() == Some(vec!["alpha-node-000".to_string()]))
                .expect("service builder for alpha-node was not created correctly");

            assert_eq!(service_alpha_node.service_id(), Some("a000".to_string()));
            assert_eq!(
                service_alpha_node.service_type(),
                Some("scabbard".to_string())
            );

            let alpha_service_args = service_alpha_node
                .arguments()
                .expect("service for alpha node has no arguments set");
            println!("alpha_service_args {:?}", alpha_service_args);
            assert!(alpha_service_args
                .iter()
                .any(|(key, value)| key == "admin-keys" && value == "$(admin-keys)"));
            assert!(alpha_service_args
                .iter()
                .any(|(key, value)| key == "peer-services" && value == "[\"a001\"]"));

            let service_beta_node = service_builders
                .iter()
                .find(|service| service.allowed_nodes() == Some(vec!["beta-node-000".to_string()]))
                .expect("service builder for beta-node was not created correctly");

            assert_eq!(service_beta_node.service_id(), Some("a001".to_string()));
            assert_eq!(
                service_beta_node.service_type(),
                Some("scabbard".to_string())
            );

            let beta_service_args = service_beta_node
                .arguments()
                .expect("service for beta node has no arguments set");
            println!("beta_service_args {:?}", beta_service_args);
            assert!(beta_service_args
                .iter()
                .any(|(key, value)| key == "admin-keys" && value == "$(admin-keys)"));
            assert!(beta_service_args
                .iter()
                .any(|(key, value)| key == "peer-services" && value == "[\"a000\"]"));
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
