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

use std::{error, fmt, io};

#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionManagerError {
    StartUpError(String),
    HeartbeatError(String),
    CreateConnectionError(String),
    SendMessageError(String),
    SendTimeoutError(String),
    ConnectionCreationError(String),
    ConnectionRemovalError(String),
    ConnectionReconnectError(String),
    Unauthorized(String),
    StatePoisoned,
}

impl error::Error for ConnectionManagerError {}

impl fmt::Display for ConnectionManagerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionManagerError::StartUpError(err) => f.write_str(err),
            ConnectionManagerError::HeartbeatError(ref s) => f.write_str(s),
            ConnectionManagerError::CreateConnectionError(ref s) => f.write_str(s),
            ConnectionManagerError::SendMessageError(ref s) => f.write_str(s),
            ConnectionManagerError::SendTimeoutError(ref s) => f.write_str(s),
            ConnectionManagerError::ConnectionCreationError(ref s) => f.write_str(s),
            ConnectionManagerError::ConnectionRemovalError(ref s) => f.write_str(s),
            ConnectionManagerError::ConnectionReconnectError(ref s) => f.write_str(s),
            ConnectionManagerError::Unauthorized(ref connection_id) => {
                write!(f, "Connection {} failed authorization", connection_id)
            }
            ConnectionManagerError::StatePoisoned => {
                f.write_str("Connection state has been poisoned")
            }
        }
    }
}

impl From<io::Error> for ConnectionManagerError {
    fn from(err: io::Error) -> Self {
        ConnectionManagerError::StartUpError(err.to_string())
    }
}

#[derive(Debug)]
pub struct AuthorizerError(pub String);

impl std::error::Error for AuthorizerError {}

impl std::fmt::Display for AuthorizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
