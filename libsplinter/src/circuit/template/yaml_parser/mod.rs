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

pub mod v0_1;

use std::fs::File;
use std::io::Read;

use super::Error;

#[derive(Deserialize, Debug)]
struct TemplateVersionGuard {
    version: String,
}

pub fn load_template(file_path: &str) -> Result<Template, Error> {
    let mut file = File::open(file_path)
        .map_err(|err| Error::new_with_source("Error opening template file", err.into()))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|err| {
        Error::new_with_source("Error reading data from template file", err.into())
    })?;

    let version_guard: TemplateVersionGuard = serde_yaml::from_slice(&data)?;
    let template = Template::deserialize(&data, &version_guard.version)?;
    Ok(template)
}

#[derive(Deserialize, Debug)]
pub enum Template {
    V0_1(v0_1::YamlCircuitCreateTemplate),
}

impl Template {
    fn deserialize(data: &[u8], version: &str) -> Result<Self, Error> {
        match version {
            "v0.1" => {
                let template = serde_yaml::from_slice::<v0_1::YamlCircuitCreateTemplate>(data)?;
                Ok(Template::V0_1(template))
            }
            _ => Err(Error::new(&format!(
                "Invalid template version: {}",
                version
            ))),
        }
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
     * Verifies load_template correctly loads a template version 0.1
     */
    #[test]
    fn test_parse_template_v0_1() {
        run_test(|test_yaml_file_path| {
            write_yaml_file(test_yaml_file_path);
            let template_version =
                load_template(test_yaml_file_path).expect("failed to load template");
            match template_version {
                Template::V0_1(template) => {
                    assert_eq!(&template.version(), "v0.1");
                    let args = template.args();
                    assert!(args.iter().any(|arg| arg.name() == "admin-keys"
                        && arg.required() == false
                        && arg.default_value() == Some("$(a:SIGNER_PUB_KEY)".into())));

                    assert_eq!(
                        &template.rules().set_management_type().management_type(),
                        "gameroom"
                    );

                    let create_services = template
                        .rules()
                        .create_services()
                        .expect("Did not parse create_services rule");
                    assert_eq!(&create_services.service_type(), "scabbard");

                    assert_eq!(&create_services.first_service(), "a000");

                    let service_args = create_services.service_args();
                    assert!(service_args
                        .iter()
                        .any(|arg| arg.key() == "admin-keys" && arg.value() == "$(admin-keys)"));
                    assert!(service_args.iter().any(|arg| arg.key() == "peer-services"
                        && arg.value() == "$(r:ALL_OTHER_SERVICES)"));
                }
            }
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
