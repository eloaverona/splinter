// Copyright 2020 Cargill Incorporated
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

use clap::ArgMatches;
use reqwest::Url;

use crate::error::CliError;

use super::file;
use super::Action;

static FILE_NAME: &str = "node_alias";

pub struct AddNodeAliasAction;

impl Action for AddNodeAliasAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let alias = match args.value_of("alias") {
            Some(alias) => alias.to_string(),
            None => return Err(CliError::ActionError("Alias is required".into())),
        };
        let endpoint = match args.value_of("endpoint") {
            Some(endpoint) => endpoint.to_string(),
            None => return Err(CliError::ActionError("Alias is required".into())),
        };

        validate_node_endpont(&endpoint)?;

        if file::fetch_key_from_file(FILE_NAME, &alias)?.is_some() {
            if args.is_present("force") {
                debug!("Overwritting existing value for alias {}", alias);
                let mut aliases = file::list_keys_from_file(FILE_NAME)?;
                aliases.insert(alias, endpoint);
                file::overwrite_file(FILE_NAME, aliases)?;
                return Ok(());
            } else {
                return Err(CliError::ActionError(format!(
                    "Alias {} already in use.",
                    alias
                )));
            }
        };

        file::write_key_value_to_file(FILE_NAME, &alias, &endpoint)?;
        Ok(())
    }
}

pub struct GetNodeAliasAction;

impl Action for GetNodeAliasAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let alias = match args.value_of("alias") {
            Some(alias) => alias,
            None => return Err(CliError::ActionError("Alias is required".into())),
        };

        match file::fetch_key_from_file(FILE_NAME, alias)? {
            Some(endpoint) => println!("{} {}", alias, endpoint),
            None => println!("Alias not found {}", alias),
        };

        Ok(())
    }
}

pub struct ListNodeAliasAction;

impl Action for ListNodeAliasAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let aliases = file::list_keys_from_file(FILE_NAME)?;
        if aliases.is_empty() {
            println!("No node alias have been set yet");
        } else {
            aliases.iter().for_each(|(alias, endpoint)| {
                println!("{} {}", alias, endpoint);
            })
        }
        Ok(())
    }
}

pub struct DeleteNodeAliasAction;

impl Action for DeleteNodeAliasAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;

        let alias = match args.value_of("alias") {
            Some(alias) => alias,
            None => return Err(CliError::ActionError("Alias is required".into())),
        };

        let mut all_aliases = file::list_keys_from_file(FILE_NAME)?;
        let value = all_aliases.remove(alias);
        if value.is_none() {
            println!("Node with alias {} not found", alias);
        } else {
            file::overwrite_file(FILE_NAME, all_aliases)?;
        }
        Ok(())
    }
}

pub fn get_endpoint_for_alias(alias: &str) -> Result<Option<String>, CliError> {
    let endpoint = file::fetch_key_from_file(FILE_NAME, alias)?;
    Ok(endpoint)
}

fn validate_node_endpont(endpoint: &str) -> Result<(), CliError> {
    if let Err(err) = Url::parse(endpoint) {
        Err(CliError::ActionError(format!(
            "{} is not a valid url: {}",
            endpoint, err
        )))
    } else {
        Ok(())
    }
}
