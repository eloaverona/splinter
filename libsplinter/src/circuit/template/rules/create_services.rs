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

use std::str::FromStr;

use super::super::{yaml_parser::v1, CircuitTemplateError, SplinterServiceBuilder};
use super::{get_argument_value, is_arg, RuleArgument, Value};

const ALL_OTHER_SERVICES: &str = "$(cs:ALL_OTHER_SERVICES)";
const NODES_ARG: &str = "$(cs:NODES)";
const PEER_SERVICES_ARG: &str = "peer-services";

pub(super) struct CreateServices {
    service_type: String,
    service_args: Vec<ServiceArgument>,
    first_service: String,
}

impl CreateServices {
    pub fn apply_rule(
        &self,
        template_arguments: &[RuleArgument],
    ) -> Result<Vec<SplinterServiceBuilder>, CircuitTemplateError> {
        let nodes = get_argument_value(NODES_ARG, template_arguments)?
            .split(',')
            .map(String::from)
            .collect::<Vec<String>>();

        if self.first_service.is_empty() {
            return Err(CircuitTemplateError::new(
                "The first_service field must be an non-empty string",
            ));
        }

        let mut service_id = self.first_service.clone();
        let mut service_builders = vec![];
        for node in nodes {
            let splinter_service_builder = SplinterServiceBuilder::new()
                .with_service_id(&service_id)
                .with_allowed_nodes(&[node])
                .with_service_type(&self.service_type);

            service_builders.push(splinter_service_builder);
            service_id = get_next_service_id(&service_id)?;
        }

        let mut new_service_args = Vec::new();
        for arg in self.service_args.iter() {
            match &arg.value {
                Value::Single(value) => {
                    if arg.key == PEER_SERVICES_ARG && value == ALL_OTHER_SERVICES {
                        service_builders = all_services(service_builders)?;
                    } else {
                        let value = if is_arg(&value) {
                            get_argument_value(&value, template_arguments)?
                        } else {
                            value.clone()
                        };
                        new_service_args.push((arg.key.clone(), value));
                    }
                }
                Value::List(values) => {
                    let vals = values
                        .iter()
                        .try_fold::<_, _, Result<_, CircuitTemplateError>>(
                            Vec::new(),
                            |mut acc, value| {
                                let value = if is_arg(&value) {
                                    get_argument_value(&value, template_arguments)?
                                } else {
                                    value.to_string()
                                };
                                acc.push(format!("\"{}\"", value));
                                Ok(acc)
                            },
                        )?;
                    new_service_args.push((arg.key.clone(), format!("[{}]", vals.join(","))));
                }
            }
        }

        service_builders = service_builders
            .into_iter()
            .map(|builder| {
                let mut service_args = builder.arguments().unwrap_or_default();
                service_args.extend(new_service_args.clone());
                builder.with_arguments(&service_args)
            })
            .collect::<Vec<SplinterServiceBuilder>>();

        Ok(service_builders)
    }
}

#[derive(Debug)]
struct ServiceArgument {
    key: String,
    value: Value,
}

impl From<v1::CreateServices> for CreateServices {
    fn from(yaml_create_services: v1::CreateServices) -> Self {
        CreateServices {
            service_type: yaml_create_services.service_type().to_string(),
            service_args: yaml_create_services
                .service_args()
                .to_owned()
                .into_iter()
                .map(ServiceArgument::from)
                .collect(),
            first_service: yaml_create_services.first_service().to_string(),
        }
    }
}

impl From<v1::ServiceArgument> for ServiceArgument {
    fn from(yaml_service_argument: v1::ServiceArgument) -> Self {
        ServiceArgument {
            key: yaml_service_argument.key().to_string(),
            value: Value::from(yaml_service_argument.value().clone()),
        }
    }
}

fn add1_char(c: char) -> Result<String, CircuitTemplateError> {
    let char_value = c as u32;
    if char_value >= 'z' as u32 {
        return Ok('0'.to_string());
    }
    let next_char = std::char::from_u32(char_value + 1)
        .ok_or_else(|| CircuitTemplateError::new("Failed to generate service id"))?;
    if !next_char.is_ascii_alphanumeric() {
        return add1_char(next_char);
    }
    Ok(next_char.to_string())
}

fn get_next_service_id(current_id: &str) -> Result<String, CircuitTemplateError> {
    generate_id(current_id, current_id.len() - 1)
}

fn generate_id(current_id: &str, index: usize) -> Result<String, CircuitTemplateError> {
    let mut next_id = current_id.to_string();
    let character = char::from_str(&next_id[index..=index])
        .map_err(|_| CircuitTemplateError::new("Failed to generate service id"))?;
    if !character.is_ascii_alphanumeric() {
        return Err(CircuitTemplateError::new(&format!(
            "The service id contains an invalid character: {}. \
             Only ASCII alphanumeric characters are allowed",
            character
        )));
    }
    let next_char = add1_char(character)?;
    next_id.replace_range(index..=index, &next_char);

    if next_char == "0" {
        if index == 0 {
            return Err(CircuitTemplateError::new(
                "Exceed number of services that can be built",
            ));
        }

        return generate_id(&next_id, index - 1);
    }
    Ok(next_id)
}

fn all_services(
    service_builders: Vec<SplinterServiceBuilder>,
) -> Result<Vec<SplinterServiceBuilder>, CircuitTemplateError> {
    let peers = service_builders.iter().map(|builder| {
        let service_id = builder.service_id()
            .ok_or_else(|| {
                error!("The service_id must be set before the service argument PEER_SERVICES can be set");
                CircuitTemplateError::new("Failed to parse template due to an internal error")
            })?;
        Ok(format!("\"{}\"", service_id))
    }).collect::<Result<Vec<String>, CircuitTemplateError>>()?;
    let services = service_builders
        .into_iter()
        .enumerate()
        .map(|(index, builder)| {
            let mut service_peers = peers.clone();
            service_peers.remove(index);
            let mut service_args = builder.arguments().unwrap_or_default();
            service_args.push((
                PEER_SERVICES_ARG.into(),
                format!("[{}]", service_peers.join(",")),
            ));
            builder.with_arguments(&service_args)
        })
        .collect::<Vec<SplinterServiceBuilder>>();
    Ok(services)
}

#[cfg(test)]
mod test {
    use super::*;

    /*
     * Test that CreateServices::apply_rules correcly sets ups
     * the services builders
     */
    #[test]
    fn test_create_service_apply_rules() {
        let create_services = make_create_service();
        let template_arguments = make_rule_arguments();

        let service_builders = create_services
            .apply_rule(&template_arguments)
            .expect("Failled to apply rules");

        assert_eq!(service_builders.len(), 2);

        assert_eq!(
            service_builders[0].allowed_nodes(),
            Some(vec!["alpha-node-000".to_string()])
        );
        assert_eq!(service_builders[0].service_id(), Some("a000".to_string()));
        assert_eq!(
            service_builders[0].service_type(),
            Some("scabbard".to_string())
        );

        let service_args = service_builders[0]
            .arguments()
            .expect("Services args were not set");
        assert_eq!(service_args.len(), 2);
        assert_eq!(
            service_args[0],
            (PEER_SERVICES_ARG.to_string(), "[\"a001\"]".to_string())
        );
        assert_eq!(
            service_args[1],
            ("admin-keys".to_string(), "[\"signer_key\"]".to_string())
        );

        assert_eq!(
            service_builders[1].allowed_nodes(),
            Some(vec!["beta-node-000".to_string()])
        );
        assert_eq!(service_builders[1].service_id(), Some("a001".to_string()));
        assert_eq!(
            service_builders[1].service_type(),
            Some("scabbard".to_string())
        );

        let service_args = service_builders[1]
            .arguments()
            .expect("Services args were not set");
        assert_eq!(service_args.len(), 2);
        assert_eq!(
            service_args[0],
            (PEER_SERVICES_ARG.to_string(), "[\"a000\"]".to_string())
        );
        assert_eq!(
            service_args[1],
            ("admin-keys".to_string(), "[\"signer_key\"]".to_string())
        );

        // test that building services succeeds:
        assert!(service_builders[0].clone().build().is_ok());
        assert!(service_builders[1].clone().build().is_ok());
    }

    fn make_create_service() -> CreateServices {
        let peer_services_arg = ServiceArgument {
            key: PEER_SERVICES_ARG.to_string(),
            value: Value::Single(ALL_OTHER_SERVICES.to_string()),
        };
        let admin_keys_arg = ServiceArgument {
            key: "admin-keys".to_string(),
            value: Value::List(vec!["$(cs:ADMIN_KEYS)".to_string()]),
        };

        CreateServices {
            service_type: "scabbard".to_string(),
            service_args: vec![peer_services_arg, admin_keys_arg],
            first_service: "a000".to_string(),
        }
    }

    fn make_rule_arguments() -> Vec<RuleArgument> {
        let admin_keys_templae_arg = RuleArgument {
            name: "cs:admin_keys".to_string(),
            required: false,
            default_value: Some("$(SIGNER_PUB_KEY)".to_string()),
            user_value: None,
        };

        let nodes_templae_arg = RuleArgument {
            name: "cs:nodes".to_string(),
            required: true,
            default_value: None,
            user_value: Some("alpha-node-000,beta-node-000".to_string()),
        };

        let signer_pub_key = RuleArgument {
            name: "signer_pub_key".to_string(),
            required: true,
            default_value: None,
            user_value: Some("signer_key".to_string()),
        };

        vec![admin_keys_templae_arg, nodes_templae_arg, signer_pub_key]
    }
}
