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

    static EXAMPLE_TEMPLATE_YAML: &[u8; 167] = br##"version: v0.1
args:
    - name: admin-keys
      required: true
      default: $(a:SIGNER_PUB_KEY)
rules:
    set-management-type:
        management-type: "gameroom" "##;

    /*
     * Verifies that Builders can be parsed from template v0.1 and correctly
     * applies the set-management-type rule
     */
    #[test]
    fn test_builds_template_v0_1() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);
            let builders = Builders::try_from_template(test_yaml_file_path, &HashMap::new())
                .expect("Error getting builders from templates");
            let circuit_create_builder = builders.create_circuit_builder();
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
