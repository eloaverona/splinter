// Copyright 2019 Cargill Incorporated
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

mod api;
mod builder;
pub mod defaults;
mod file;
mod payload;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use clap::{ArgMatches, Values};
use regex::Regex;
use reqwest::Url;
use splinter::admin::messages::{
    AuthorizationType, CreateCircuit, DurabilityType, PersistenceType, RouteType, SplinterNode,
    SplinterService,
};

use crate::error::CliError;

use super::Action;
use builder::MessageBuilder;

pub struct CircuitCreateAction;

impl Action for CircuitCreateAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = args.value_of("url").unwrap_or("http://localhost:8085");
        let key = args
            .value_of("private_key_file")
            .unwrap_or("./splinter.priv");
        if let Some(path) = args.value_of("path") {
            create_circuit_proposal(url, key, path)
        } else {
            let mut nodes = match args.values_of("node") {
                Some(nodes) => nodes,
                None => return Err(CliError::ActionError("Path is required".into())),
            };

            let mut builder = MessageBuilder::new();

            for node in nodes {
                builder.add_node(node);
            }

            // let members: Result<Vec<SplinterNode>, CliError> =
            //     nodes.try_fold(Vec::new(), |mut acc, node| {
            //         let splinter_node = make_splinter_node(&node)?;
            //         acc.push(splinter_node);
            //         Ok(acc)
            //     });

            let mut services = match args.values_of("service") {
                Some(mut services) => services, //parse_service_arg(&mut services)?,
                None => return Err(CliError::ActionError("Service is required".into())),
            };

            for service in services {
                let (service_id, allowed_nodes) = parse_service(service)?;
                builder.add_service(&service_id, &allowed_nodes);
            }


            if let Some(service_arguments) = args.values_of("service_argument") {
                for service_argument in service_arguments {
                    let (service_id_match, argument) = parse_service_argument(service_argument)?;
                    builder.apply_service_arguments(&service_id_match, &argument);
                }

            }
            // let mut service_arguments = match args.values_of("service_argument") {
            //     Some(mut arguments) => arguments, //parse_service_argument(&mut arguments)?,
            //     None => return Err(CliError::ActionError("Service is required".into())),
            // };
            //
            // for service_argument in service_arguments {
            //     let (service_id_match, argument) = parse_service_argument(service_argument)?;
            //     builder.apply_service_arguments(&service_id_match, &argument);
            // }

            let mut auth_type = args.value_of("authorization_type").unwrap_or("trust");

            if let Some(management_type) = args.value_of("management_type") {
                builder.set_management_type(management_type);
            }
            // let mut management_type = match args.value_of("management_type") {
            //     Some(val) => val.to_string(),
            //     None => {
            //         return Err(CliError::ActionError(
            //             "Management-type not provided and no default set".into(),
            //         ));
            //         // match defaults::get_default_management_type()?{
            //         //     Some(val) => val,
            //         // None => return Err(CliError::ActionError(
            //         //     "Management-type not provided and no default set".into(),
            //         // ))
            //         // }
            //     }
            // };

            if let Some(service_types) = args.values_of("service_type") {
                for service_type_arg in service_types {
                    let (service_id_match, service_type) =
                        parse_sercive_type_argument(service_type_arg)?;
                    builder.apply_service_type(&service_id_match, &service_type);
                }
            }
            // let mut service_types = match args.values_of("service_type") {
            //     Some(mut service_type) => service_type //parse_sercive_type_arg(&mut service_type)?,
            //     None => {
            //         return Err(CliError::ActionError(
            //             "Service-type not provided and no default set".into(),
            //         ));
            //         //         let default_val = defaults::get_default_service_type()?;
            //         match default_val {
            //             Some(val) => {
            //                 let service_id_match = Regex::new(".*").map_err(|_| {
            //                     CliError::ActionError("Failed to set service-type".into())
            //                 })?;
            //                 vec![(service_id_match, val)]
            //             }
            //             None => {
            //                 return Err(CliError::ActionError(
            //                     "Service-type not provided and no default set".into(),
            //                 ))
            //             }
            //         }
            //     }
            // };

            // let services_with_type = assign_type_to_service(&services, &service_types)?;
            //
            // let splinter_services = make_service_roster(&services_with_type, &service_argument);
            // warn!("nodes {:?}", members);
            // warn!("service_types {:?}", service_types);
            // warn!("services {:?}", splinter_services);
            // warn!("services_with_type {:?}", services_with_type);
            // warn!("service_argument {:?}", service_argument);
            // warn!("management_type {:?}", management_type);
            // warn!("auth_type {:?}", auth_type);
            //
            // let create_circuit = CreateCircuit {
            //     circuit_id: "adasda".to_string(),
            //     roster: splinter_services,
            //     members: members?,
            //     authorization_type: AuthorizationType::Trust,
            //     persistence: PersistenceType::Any,
            //     durability: DurabilityType::NoDurability,
            //     routes: RouteType::Any,
            //     circuit_management_type: management_type,
            //     application_metadata: vec![],
            // };

            let create_circuit = builder.build().expect("Failed to build");

            let client = api::SplinterRestClient::new(url);
            let requester_node = client.fetch_node_id()?;
            let private_key_hex = read_private_key(key)?;

            // let proposal_file = File::open(proposal_path).map_err(|err| {
            //     CliError::EnvironmentError(format!("Unable to open {}: {}", proposal_path, err))
            // })?;
            //
            // let create_request: CreateCircuit = serde_yaml::from_reader(proposal_file).map_err(|err| {
            //     CliError::EnvironmentError(format!("Unable to parse {}: {}", proposal_path, err))
            // })?;

            let signed_payload =
                payload::make_signed_payload(&requester_node, &private_key_hex, create_circuit)?;

            client.submit_admin_payload(signed_payload)?;

            //warn!("services_with_type {:?}", services_with_type);

            Ok(())
        }
    }
}

// fn parse_service_arg(service_type_values: &mut Values) -> Result<Vec<(String, Vec<String>)>, CliError> {
//     service_type_values.try_fold(Vec::new(), |mut acc, val| {
//         let mut iter = val.split("::");
//
//         let service_id = iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_type not valid {}", val)))?
//             .to_string();
//
//         let allowed_nodes = iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("allowed nodes not valid {}", val)))?
//             .split(",")
//             .map(String::from)
//             .collect::<Vec<String>>();
//         acc.push((service_id, allowed_nodes));
//         Ok(acc)
//     })
// }

fn parse_service(service: &str) -> Result<(String, Vec<String>), CliError> {
    //service_type_values.try_fold(Vec::new(), |mut acc, val| {
    let mut iter = service.split("::");

    let service_id = iter
        .next()
        .ok_or_else(|| CliError::ActionError(format!("service_type not valid {}", service)))?
        .to_string();

    let allowed_nodes = iter
        .next()
        .ok_or_else(|| CliError::ActionError(format!("allowed nodes not valid {}", service)))?
        .split(",")
        .map(String::from)
        .collect::<Vec<String>>();

    Ok((service_id, allowed_nodes))
    //     acc.push((service_id, allowed_nodes));
    //     Ok(acc)
    // })
}

fn parse_service_argument(service_argument: &str) -> Result<(String, (String, String)), CliError> {
    //service_arguments_values.try_fold(Vec::new(), |mut acc, val| {
    let mut iter = service_argument.split("::");

    let service_id = iter
        .next()
        .ok_or_else(|| {
            CliError::ActionError(format!("service_argument not valid {}", service_argument))
        })?
        .to_string();

    // let re = if service_id.starts_with("*") {
    //     Regex::new(&format!(".{}", service_id)).map_err(|_| {
    //         CliError::ActionError(format!("service_id is not valid {}", service_id))
    //     })
    // } else {
    //     Regex::new(&service_id).map_err(|_| {
    //         CliError::ActionError(format!("service_id is not valid {}", service_id))
    //     })
    // }?;

    let arguments = iter
        .next()
        .ok_or_else(|| {
            CliError::ActionError(format!("service_argument not valid {}", service_argument))
        })?
        .to_string();
    let mut argument_iter = arguments.split("=");
    let key = argument_iter
        .next()
        .ok_or_else(|| {
            CliError::ActionError(format!("service_argument not valid {}", service_argument))
        })?
        .to_string();
    let value = argument_iter
        .next()
        .ok_or_else(|| {
            CliError::ActionError(format!("service_argument not valid {}", service_argument))
        })?
        .to_string();
    // .split(",")
    // .map(String::from)
    // .collect();
    Ok((service_id, (key, value)))
    //     acc.push((service_id, (key, value)));
    //     Ok(acc)
    // })
}

// fn parse_service_argument(
//     service_arguments_values: &mut Values,
// ) -> Result<Vec<(String, (String, String))>, CliError> {
//     service_arguments_values.try_fold(Vec::new(), |mut acc, val| {
//         let mut iter = val.split("::");
//
//         let service_id = iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_argument not valid {}", val)))?
//             .to_string();
//
//         // let re = if service_id.starts_with("*") {
//         //     Regex::new(&format!(".{}", service_id)).map_err(|_| {
//         //         CliError::ActionError(format!("service_id is not valid {}", service_id))
//         //     })
//         // } else {
//         //     Regex::new(&service_id).map_err(|_| {
//         //         CliError::ActionError(format!("service_id is not valid {}", service_id))
//         //     })
//         // }?;
//
//         let arguments = iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_argument not valid {}", val)))?
//             .to_string();
//         let mut argument_iter = arguments.split("=");
//         let key = argument_iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_argument not valid {}", val)))?
//             .to_string();
//         let value = argument_iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_argument not valid {}", val)))?
//             .to_string();
//             // .split(",")
//             // .map(String::from)
//             // .collect();
//         acc.push((service_id, (key, value)));
//         Ok(acc)
//     })
// }
fn parse_sercive_type_argument(service_type: &str) -> Result<(String, String), CliError> {
    //service_type_values.try_fold(Vec::new(), |mut acc, val| {
    let mut iter = service_type.split("::");

    let service_id = iter
        .next()
        .ok_or_else(|| CliError::ActionError(format!("service_type not valid {}", service_type)))?
        .to_string();

    // let re = if service_id.starts_with("*") {
    //     Regex::new(&format!(".{}", service_id)).map_err(|_| {
    //         CliError::ActionError(format!("service_id is not valid {}", service_id))
    //     })
    // } else {
    //     Regex::new(&service_id).map_err(|_| {
    //         CliError::ActionError(format!("service_id is not valid {}", service_id))
    //     })
    // }?;

    let service_type = iter
        .next()
        .ok_or_else(|| CliError::ActionError(format!("service_type not valid {}", service_type)))?
        .to_string();
    Ok((service_id, service_type))
    //     acc.push((re, service_type));
    //     Ok(acc)
    // })
}

// fn parse_sercive_type_arg(
//     service_type_values: &mut Values,
// ) -> Result<Vec<(Regex, String)>, CliError> {
//     service_type_values.try_fold(Vec::new(), |mut acc, val| {
//         let mut iter = val.split("::");
//
//         let service_id = iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_type not valid {}", val)))?
//             .to_string();
//
//         let re = if service_id.starts_with("*") {
//             Regex::new(&format!(".{}", service_id)).map_err(|_| {
//                 CliError::ActionError(format!("service_id is not valid {}", service_id))
//             })
//         } else {
//             Regex::new(&service_id).map_err(|_| {
//                 CliError::ActionError(format!("service_id is not valid {}", service_id))
//             })
//         }?;
//
//         let service_type = iter
//             .next()
//             .ok_or_else(|| CliError::ActionError(format!("service_type not valid {}", val)))?
//             .to_string();
//         acc.push((re, service_type));
//         Ok(acc)
//     })
// }

fn assign_type_to_service(
    services: &Vec<(String, String)>,
    services_type: &Vec<(Regex, String)>,
) -> Result<HashMap<String, Vec<(String, String)>>, CliError> {
    services.iter().try_fold(
        HashMap::<String, Vec<(String, String)>>::new(),
        |mut acc, (service_id, allowed_nodes)| {
            let service_type = services_type
                .iter()
                .find_map(|(service_id_match, service_type)| {
                    if service_id_match.is_match(&service_id) {
                        Some(service_type)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    CliError::ActionError(format!(
                        "No service type match for service {}",
                        service_id
                    ))
                })?;

            match acc.get_mut(service_type) {
                Some(services) => {
                    services.push((service_id.to_string(), allowed_nodes.to_string()))
                }
                None => {
                    acc.insert(
                        service_type.to_string(),
                        vec![(service_id.to_string(), allowed_nodes.to_string())],
                    );
                }
            }
            Ok(acc)
        },
    )
}

fn make_service_roster(
    services: &HashMap<String, Vec<(String, String)>>,
    service_args: &Vec<(Regex, (String, Vec<String>))>,
) -> Vec<SplinterService> {
    // services.iter().try_fold(Vec::new(), |mut acc, (service_id, allowed_nodes)| {
    //     let service_type = services_type.iter().find_map(|(service_id_match, service_type)| {
    //         let re = Regex::new(service_id_match).unwrap();
    //         if re.is_match(service_id) {
    //             Some(service_type)
    //         } else {
    //             None
    //         }
    //     });
    // })
    services.keys().fold(Vec::new(), |mut acc, (service_type)| {
        if let Some(services) = services.get(service_type) {
            let peer_services = services
                .iter()
                .map(|(service_id, _)| service_id.to_string())
                .collect::<Vec<String>>();
            let mut splinter_services = services
                .iter()
                .enumerate()
                .map(|(index, (service_id, allowed_nodes))| {
                    warn!("peer_services {:?}", peer_services);
                    let mut peers = peer_services.clone();
                    peers.remove(index);
                    warn!("PEERS {:?}", peers);
                    warn!("INDEX {:?}", index);
                    let mut args = service_args.iter().fold(Vec::new(), |mut acc, mut args| {
                        if args.0.is_match(service_id) {
                            let key = (args.1).0.to_owned();
                            let val = format!("{:?}", (args.1).1);
                            acc.push((key, val));
                        };
                        acc
                    });
                    args.push(("peer_services".into(), format!("{:?}", peers)));

                    SplinterService {
                        service_id: service_id.to_string(),
                        service_type: service_type.to_string(),
                        allowed_nodes: vec![allowed_nodes.into()],
                        arguments: args,
                    }
                })
                .collect::<Vec<SplinterService>>();
            acc.append(&mut splinter_services);
        }
        acc
    })
}

fn make_splinter_node(value: &str) -> Result<SplinterNode, CliError> {
    match Url::parse(value) {
        Ok(url) => {
            let host = match url.host_str() {
                Some(host) => host,
                None => {
                    return Err(CliError::ActionError(format!(
                        "Invalid node endpoint: {}",
                        value
                    )))
                }
            };
            let port = match url.port_or_known_default() {
                Some(port) => port,
                None => {
                    return Err(CliError::ActionError(format!(
                        "Invalid node endpoint: {}",
                        value
                    )))
                }
            };
            Ok(SplinterNode {
                node_id: format!("{}_{}", host, port),
                endpoint: value.to_string(),
            })
        }
        Err(err) => Err(CliError::ActionError(format!(
            "Invalid node endpoint or node alias has not been set: {}",
            value
        ))),
        //match node::get_endpoint_for_alias(value)? {
        //     Some(endpoint) => Ok(SplinterNode {
        //         node_id: value.to_string(),
        //         endpoint: endpoint.to_string(),
        //     }),
        //     None => Err(CliError::ActionError(format!(
        //         "Invalid node endpoint or node alias has not been set: {}",
        //         value
        //     ))),
        // },
    }
}

// fn generate_node_id(enpoint: &str) -> String {
//
// }

fn create_circuit_proposal(
    url: &str,
    private_key_file: &str,
    proposal_path: &str,
) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let requester_node = client.fetch_node_id()?;
    let private_key_hex = read_private_key(private_key_file)?;

    let proposal_file = File::open(proposal_path).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to open {}: {}", proposal_path, err))
    })?;

    let create_request: CreateCircuit = serde_yaml::from_reader(proposal_file).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to parse {}: {}", proposal_path, err))
    })?;

    let signed_payload =
        payload::make_signed_payload(&requester_node, &private_key_hex, create_request)?;

    client.submit_admin_payload(signed_payload)
}

fn create_circuit_proposal_no_file(
    url: &str,
    private_key_file: &str,
    proposal_path: &str,
) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let requester_node = client.fetch_node_id()?;
    let private_key_hex = read_private_key(private_key_file)?;

    let proposal_file = File::open(proposal_path).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to open {}: {}", proposal_path, err))
    })?;

    let create_request: CreateCircuit = serde_yaml::from_reader(proposal_file).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to parse {}: {}", proposal_path, err))
    })?;

    let signed_payload =
        payload::make_signed_payload(&requester_node, &private_key_hex, create_request)?;

    client.submit_admin_payload(signed_payload)
}

/// Reads a private key from the given file name.
pub fn read_private_key(file_name: &str) -> Result<String, CliError> {
    let mut file = File::open(file_name).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to open {}: {}", file_name, err))
    })?;

    let mut buf = String::new();

    file.read_to_string(&mut buf).map_err(|err| {
        CliError::EnvironmentError(format!("Unable to read {}: {}", file_name, err))
    })?;

    Ok(buf)
}

pub(self) enum Vote {
    Accept,
    Reject,
}

pub(self) struct CircuitVote {
    circuit_id: String,
    circuit_hash: String,
    vote: Vote,
}

pub struct CircuitVoteAction;

impl Action for CircuitVoteAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let url = args.value_of("url").unwrap_or("http://localhost:8085");
        let key = args.value_of("private_key_file").unwrap_or("splinter");
        let circuit_id = match args.value_of("circuit_id") {
            Some(circuit_id) => circuit_id,
            None => return Err(CliError::ActionError("Circuit id is required".into())),
        };

        // accept or reject must be present
        let vote = {
            if args.is_present("accept") {
                Vote::Accept
            } else {
                Vote::Reject
            }
        };

        vote_on_circuit_proposal(url, key, circuit_id, vote)
    }
}

fn vote_on_circuit_proposal(
    url: &str,
    key: &str,
    circuit_id: &str,
    vote: Vote,
) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let private_key_hex = read_private_key(key)?;

    let requester_node = client.fetch_node_id()?;
    let proposal = client.fetch_proposal(circuit_id)?;

    if let Some(proposal) = proposal {
        let circuit_vote = CircuitVote {
            circuit_id: circuit_id.into(),
            circuit_hash: proposal.circuit_hash,
            vote,
        };

        let signed_payload =
            payload::make_signed_payload(&requester_node, &private_key_hex, circuit_vote)?;

        client.submit_admin_payload(signed_payload)
    } else {
        Err(CliError::ActionError(format!(
            "Proposal for {} does not exist",
            circuit_id
        )))
    }
}

pub struct CircuitListAction;

impl Action for CircuitListAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let url = args.value_of("url").unwrap_or("http://127.0.0.1:8080");

        let filter = args.value_of("member");

        list_circuits(url, filter)
    }
}

fn list_circuits(url: &str, filter: Option<&str>) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);

    let circuits = client.list_circuits(filter)?;
    println!(
        "{0: <80} | {1: <30}",
        "CIRCUIT ID", "CIRCUIT MANAGEMENT TYPE",
    );
    println!("{}", "-".repeat(110));
    circuits.data.iter().for_each(|circuit| {
        println!(
            "{0: <80} | {1: <30}",
            circuit.id, circuit.circuit_management_type,
        );
    });
    Ok(())
}

pub struct CircuitShowAction;

impl Action for CircuitShowAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let url = args.value_of("url").unwrap_or("http://127.0.0.1:8080");
        let circuit_id = args
            .value_of("circuit")
            .ok_or_else(|| CliError::ActionError("Circuit name must be provided".to_string()))?;

        // A value should always be passed because a default is defined
        let format = args.value_of("format").expect("format was not provided");

        show_circuit(url, circuit_id, format)
    }
}

fn show_circuit(url: &str, circuit_id: &str, format: &str) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);
    let circuit = client.fetch_circuit(circuit_id)?;
    let mut print_circuit = false;
    let mut print_proposal = false;
    if let Some(circuit) = circuit {
        print_circuit = true;
        match format {
            "json" => println!(
                "\n {}",
                serde_json::to_string(&circuit).map_err(|err| CliError::ActionError(format!(
                    "Cannot format circuit into json: {}",
                    err
                )))?
            ),
            // default is yaml
            _ => println!(
                "{}",
                serde_yaml::to_string(&circuit).map_err(|err| CliError::ActionError(format!(
                    "Cannot format circuit into yaml: {}",
                    err
                )))?
            ),
        }
    }

    let proposal = client.fetch_proposal(circuit_id)?;

    if let Some(proposal) = proposal {
        print_proposal = true;
        match format {
            "json" => println!(
                "\n {}",
                serde_json::to_string(&proposal).map_err(|err| CliError::ActionError(format!(
                    "Cannot format proposal into json: {}",
                    err
                )))?
            ),
            // default is yaml
            _ => println!(
                "{}",
                serde_yaml::to_string(&proposal).map_err(|err| CliError::ActionError(format!(
                    "Cannot format proposal into yaml: {}",
                    err
                )))?
            ),
        }
    }

    if !print_circuit && !print_proposal {
        return Err(CliError::ActionError(format!(
            "Proposal for {} does not exist",
            circuit_id
        )));
    }

    Ok(())
}

pub struct CircuitProposalsAction;

impl Action for CircuitProposalsAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let url = args.value_of("url").unwrap_or("http://127.0.0.1:8080");

        let filter = args.value_of("management_type");

        list_proposals(url, filter)
    }
}

fn list_proposals(url: &str, filter: Option<&str>) -> Result<(), CliError> {
    let client = api::SplinterRestClient::new(url);

    let proposals = client.list_proposals(filter)?;
    println!(
        "{0: <80} | {1: <30}",
        "CIRCUIT ID", "CIRCUIT MANAGEMENT TYPE",
    );
    println!("{}", "-".repeat(110));
    proposals.data.iter().for_each(|proposal| {
        println!(
            "{0: <80} | {1: <30}",
            proposal.circuit_id, proposal.circuit.circuit_management_type,
        );
    });
    Ok(())
}
