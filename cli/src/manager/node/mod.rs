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

use reqwest::Url;

use crate::store::node::{FileBackedNodeStore, NodeStore};

pub use crate::store::node::Node;
pub use error::NodeManagerError;

pub struct NodeManager {
    store: Box<dyn NodeStore>,
}

impl Default for NodeManager {
    fn default() -> Self {
        NodeManager {
            store: Box::new(FileBackedNodeStore::default()),
        }
    }
}

impl NodeManager {
    pub fn get_node(&self, alias: &str) -> Result<Option<Node>, NodeManagerError> {
        let node = self.store.get_node(alias)?;
        Ok(node)
    }

    pub fn list_nodes(&self) -> Result<Vec<Node>, NodeManagerError> {
        let nodes = self.store.list_nodes()?;
        Ok(nodes)
    }

    pub fn add_node(&self, node: &Node, overwrite: bool) -> Result<(), NodeManagerError> {
        validate_node_endpont(&node.endpoint())?;
        if !overwrite && self.store.get_node(&node.alias())?.is_some() {
            return Err(NodeManagerError::DuplicatedValue(format!(
                "Alias {} is already in use",
                node.alias()
            )));
        }

        self.store.add_node(node)?;
        Ok(())
    }

    pub fn delete_node(&self, alias: &str) -> Result<(), NodeManagerError> {
        self.store.delete_node(alias)?;
        Ok(())
    }
}

fn validate_node_endpont(endpoint: &str) -> Result<(), NodeManagerError> {
    if let Err(err) = Url::parse(endpoint) {
        Err(NodeManagerError::InvalidEndpoint(format!(
            "{} is not a valid url: {}",
            endpoint, err
        )))
    } else {
        Ok(())
    }
}
