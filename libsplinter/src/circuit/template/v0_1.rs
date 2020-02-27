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
use super::{Builders, CreateCircuitBuilder, SplinterServiceBuilder};

const ALL_OTHER_SERVICES: &str = "$(cs:ALL_OTHER_SERVICES)";
const NODES_ARG: &str = "$(cs:NODES)";
const SIGNER_PUB_KEY: &str = "$(SIGNER_PUB_KEY)";

const PEER_SERVICES_ARG: &str = "peer-services";

pub struct CircuitCreateTemplate {
    _version: String,
    args: Vec<RuleArgument>,
    rules: Rules,
}

impl CircuitCreateTemplate {
    pub fn apply_rules(
        &self,
        builders: &mut Builders,
        arguments: &HashMap<String, String>,
    ) -> Result<(), Error> {
        self.rules.apply_rules(builders, arguments, &self.args)
    }
}

impl From<v0_1::YamlCircuitCreateTemplate> for CircuitCreateTemplate {
    fn from(create_circuit_templase: v0_1::YamlCircuitCreateTemplate) -> Self {
        CircuitCreateTemplate {
            _version: create_circuit_templase.version(),
            args: create_circuit_templase
                .args()
                .into_iter()
                .map(RuleArgument::from)
                .collect(),
            rules: Rules::from(create_circuit_templase.rules()),
        }
    }
}

struct RuleArgument {
    name: String,
    required: bool,
    default_value: Option<String>,
}

impl From<v0_1::YamlRuleArgument> for RuleArgument {
    fn from(arguments: v0_1::YamlRuleArgument) -> Self {
        RuleArgument {
            name: arguments.name(),
            required: arguments.required(),
            default_value: arguments.default_value(),
        }
    }
}

struct Rules {
    set_management_type: CircuitManagement,
    create_services: Option<CreateServices>,
}

impl Rules {
    fn apply_rules(
        &self,
        builders: &mut Builders,
        arguments: &HashMap<String, String>,
        template_arguments: &[RuleArgument],
    ) -> Result<(), Error> {
        let mut service_builders = builders.service_builders();

        let create_service_builder = self
            .set_management_type
            .apply_rule(builders.create_circuit_builder())?;

        if let Some(create_services) = &self.create_services {
            service_builders = create_services.apply_rule(arguments, template_arguments)?;
        }
        builders.set_create_circuit_builder(create_service_builder);
        builders.set_service_builders(service_builders);
        Ok(())
    }
}

impl From<v0_1::YamlRules> for Rules {
    fn from(rules: v0_1::YamlRules) -> Self {
        Rules {
            set_management_type: CircuitManagement::from(rules.set_management_type()),
            create_services: rules.create_services().map(CreateServices::from),
        }
    }
}

struct CircuitManagement {
    management_type: String,
}

impl CircuitManagement {
    fn apply_rule(&self, builder: CreateCircuitBuilder) -> Result<CreateCircuitBuilder, Error> {
        Ok(builder.with_circuit_management_type(&self.management_type))
    }
}

impl From<v0_1::YamlCircuitManagement> for CircuitManagement {
    fn from(yaml_circuit_management: v0_1::YamlCircuitManagement) -> Self {
        CircuitManagement {
            management_type: yaml_circuit_management.management_type(),
        }
    }
}

struct CreateServices {
    service_type: String,
    service_args: Vec<ServiceArgument>,
    first_service: String,
}

impl CreateServices {
    fn apply_rule(
        &self,
        args: &HashMap<String, String>,
        template_arguments: &[RuleArgument],
    ) -> Result<Vec<SplinterServiceBuilder>, Error> {
        let nodes = get_argument_value(NODES_ARG, args, template_arguments)?
            .ok_or_else(|| {
                Error::new(&format!(
                    "Invalid template. Argument {} was expected but not provided",
                    NODES_ARG
                ))
            })?
            .split(",")
            .map(String::from)
            .collect::<Vec<String>>();

        let mut service_id = self.first_service.clone();
        let mut service_builders = vec![];
        for node in nodes {
            let mut splinter_service_builder = SplinterServiceBuilder::new()
                .with_service_id(&service_id)
                .with_allowed_nodes(&vec![node])
                .with_service_type(&self.service_type);

            service_builders.push(splinter_service_builder);
            service_id = get_next_service_id(&service_id)?;
        }
        println!("self.service_args {:?}", self.service_args);

        let mut new_service_args = Vec::new();
        for arg in self.service_args.iter() {
            if arg.key == PEER_SERVICES_ARG && arg.value == ALL_OTHER_SERVICES {
                service_builders = all_services(service_builders)?;
            } else {
                let value = match get_argument_value(&arg.value, args, template_arguments)? {
                    Some(val) => val,
                    None => arg.value.clone(),
                };
                new_service_args.push((arg.key.clone(), value))
            }
        }
        println!("service_args {:?}", new_service_args);
        service_builders = service_builders
            .into_iter()
            .map(|builder| {
                let mut service_args = builder.arguments().unwrap_or_default();
                println!("already set service_args {:?}", service_args);

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
    value: String,
}

impl From<v0_1::YamlCreateServices> for CreateServices {
    fn from(yaml_create_services: v0_1::YamlCreateServices) -> Self {
        CreateServices {
            service_type: yaml_create_services.service_type(),
            service_args: yaml_create_services
                .service_args()
                .into_iter()
                .map(ServiceArgument::from)
                .collect(),
            first_service: yaml_create_services.first_service(),
        }
    }
}

impl From<v0_1::YamlServiceArgument> for ServiceArgument {
    fn from(yaml_service_argument: v0_1::YamlServiceArgument) -> Self {
        ServiceArgument {
            key: yaml_service_argument.key(),
            value: yaml_service_argument.value(),
        }
    }
}

fn get_next_service_id(current_id: &str) -> Result<String, Error> {
    let alphanumeric = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let mut next_id = current_id.to_string();
    for (char_index, char) in current_id.char_indices().rev() {
        let index = alphanumeric.find(char).ok_or_else(|| {
            Error::new("The field first_service must contain only valid base62 characters")
        })?;
        match alphanumeric.get(index + 1..index + 2) {
            Some(sub_str) => {
                let mut next_id_sub_string = next_id.get(char_index + 1..).unwrap_or_default();
                let new_sub_string = format!("{}{}", sub_str, next_id_sub_string);
                next_id.replace_range(char_index.., &new_sub_string);
                return Ok(next_id);
            }
            None => {
                let mut next_id_sub_string = next_id.get(char_index + 1..).unwrap_or_default();
                let new_sub_string =
                    format!("{}{}", alphanumeric[0..1].to_string(), next_id_sub_string);
                next_id.replace_range(char_index.., &new_sub_string);
            }
        }
    }

    return Err(Error::new("Exceed number of services that can be built"));
}

fn all_services(
    service_builders: Vec<SplinterServiceBuilder>,
) -> Result<Vec<SplinterServiceBuilder>, Error> {
    let peers = service_builders.iter().try_fold::<_, _, Result<Vec<String>, Error>>(Vec::new(), |mut acc, builder| {
        let service_id = builder.service_id()
            .ok_or_else(|| {
                error!("The service_id must be set before the service argument PEER_SERVICES can be set");
                Error::new("Failed to parse template due to an internal error")
            })?;
        acc.push(service_id);
        Ok(acc)
    })?;
    let services = service_builders
        .into_iter()
        .enumerate()
        .map(|(index, builder)| {
            let mut service_peers = peers.clone();
            service_peers.remove(index);
            let mut service_args = builder.arguments().unwrap_or_default();
            service_args.push((PEER_SERVICES_ARG.into(), format!("{:?}", service_peers)));
            builder.with_arguments(&service_args)
        })
        .collect::<Vec<SplinterServiceBuilder>>();
    Ok(services)
}

fn get_argument_value(
    key: &str,
    args: &HashMap<String, String>,
    template_arguments: &[RuleArgument],
) -> Result<Option<String>, Error> {
    let value = match template_arguments.iter().find(|arg| arg.name == key) {
        Some(arg) => match args.get(key) {
            Some(val) => Some(val.to_string()),
            None => {
                if arg.required {
                    return Err(Error::new(&format!(
                        "Argument {} is required but was not provided",
                        key
                    )));
                } else {
                    let default_value = arg
                        .default_value
                        .as_ref()
                        .ok_or_else(|| {
                            Error::new(&format!(
                                "Argument {} was not provided and no default value is set",
                                key
                            ))
                        })?
                        .clone();
                    println!("default_value {}", default_value);
                    println!("is  SIGNER_PUB_KEY {}", default_value == SIGNER_PUB_KEY);

                    let val = if default_value == SIGNER_PUB_KEY {
                        match args.get(SIGNER_PUB_KEY) {
                            Some(val) => val.to_string(),
                            None => return Err(Error::new("Signer public key was not provided")),
                        }
                    } else {
                        default_value.to_string()
                    };
                    Some(val)
                }
            }
        },
        None => None,
    };

    Ok(value)
}
