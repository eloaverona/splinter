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

use super::Action;
use crate::error::CliError;
use crate::manager::{Node, NodeManager, NodeManagerError};
use clap::ArgMatches;

pub struct AddNodeAliasAction;

impl Action for AddNodeAliasAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let args = arg_matches.ok_or_else(|| CliError::RequiresArgs)?;
        let alias = match args.value_of("alias") {
            Some(alias) => alias,
            None => return Err(CliError::ActionError("Alias is required".into())),
        };
        let endpoint = match args.value_of("endpoint") {
            Some(endpoint) => endpoint,
            None => return Err(CliError::ActionError("Endpoint is required".into())),
        };
        let node = Node::new(alias, endpoint);

        let node_manager = NodeManager::default();

        node_manager.add_node(&node, args.is_present("force"))?;

        Ok(())
    }
}

pub struct ListNodeAliasAction;

impl Action for ListNodeAliasAction {
    fn run<'a>(&mut self, _: Option<&ArgMatches<'a>>) -> Result<(), CliError> {
        let node_manager = NodeManager::default();

        let nodes = node_manager.list_nodes()?;

        if nodes.is_empty() {
            println!("No node alias have been set yet");
        } else {
            nodes.iter().for_each(|node| {
                println!("{} {}", node.alias(), node.endpoint());
            })
        }
        Ok(())
    }
}

impl From<NodeManagerError> for CliError {
    fn from(err: NodeManagerError) -> Self {
        CliError::ActionError(format!("Failed to perform node operation: {}", err))
    }
}
