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

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[cfg(feature = "config-command-line")]
#[macro_use]
extern crate clap;

#[cfg(feature = "generate-certs")]
mod certs;
mod config;
mod daemon;
mod registry_config;
mod routes;

use flexi_logger::{style, DeferredNow, LogSpecBuilder, Logger};
use log::Record;

#[cfg(feature = "generate-certs")]
use crate::certs::{make_ca_cert, make_ca_signed_cert, write_file, CertError};
#[cfg(feature = "config-command-line")]
use crate::config::CommandLineConfig;
#[cfg(feature = "config-default")]
use crate::config::DefaultConfig;
#[cfg(feature = "config-env-var")]
use crate::config::EnvVarConfig;
#[cfg(feature = "default")]
use crate::config::PartialConfigBuilder;
#[cfg(feature = "config-toml")]
use crate::config::TomlConfig;
use crate::config::{Config, ConfigBuilder, ConfigError};
use crate::daemon::{SplinterDaemonBuilder, StartError};
use clap::{clap_app, crate_version};
use clap::{Arg, ArgMatches};
#[cfg(feature = "generate-certs")]
use openssl::error::ErrorStack;
use splinter::transport::raw::RawTransport;
use splinter::transport::tls::{TlsInitError, TlsTransport};
use splinter::transport::Transport;
#[cfg(feature = "generate-certs")]
use tempdir::TempDir;

use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::thread;

fn create_config(_toml_path: Option<&str>, _matches: ArgMatches) -> Result<Config, UserError> {
    #[cfg(feature = "default")]
    let mut builder = ConfigBuilder::new();
    #[cfg(not(feature = "default"))]
    let builder = ConfigBuilder::new();

    #[cfg(feature = "config-command-line")]
    {
        let command_line_config = CommandLineConfig::new(_matches)
            .map_err(UserError::ConfigError)?
            .build();
        builder = builder.with_partial_config(command_line_config);
    }

    #[cfg(feature = "config-toml")]
    {
        if let Some(file) = _toml_path {
            let toml_string = fs::read_to_string(file).map_err(|err| ConfigError::ReadError {
                file: String::from(file),
                err,
            })?;
            let toml_config = TomlConfig::new(toml_string, String::from(file))
                .map_err(UserError::ConfigError)?
                .build();
            builder = builder.with_partial_config(toml_config);
        }
    }

    #[cfg(feature = "config-env-var")]
    {
        let env_config = EnvVarConfig::new().build();
        builder = builder.with_partial_config(env_config);
    }

    #[cfg(feature = "config-default")]
    {
        let default_config = DefaultConfig::new().build();
        builder = builder.with_partial_config(default_config);
    }
    builder
        .build()
        .map_err(|e| UserError::MissingArgument(e.to_string()))
}

// format for logs
pub fn log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] T[{:?}] {} [{}] {}",
        now.now().format("%Y-%m-%d %H:%M:%S%.3f"),
        thread::current().name().unwrap_or("<unnamed>"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        style(level, &record.args()),
    )
}

fn main() {
    let app = clap_app!(splinterd =>
        (version: crate_version!())
        (about: "Splinter Daemon")
        (@arg config: -c --config +takes_value)
        (@arg node_id: --("node-id") +takes_value
          "Unique ID for the node ")
        (@arg storage: --("storage") +takes_value
          "Storage type used for the node; defaults to yaml")
        (@arg transport: --("transport") +takes_value
          "Transport type for sockets, either raw or tls")
        (@arg network_endpoint: -n --("network-endpoint") +takes_value
          "Endpoint to connect to the network, tcp://ip:port")
        (@arg service_endpoint: --("service-endpoint") +takes_value
          "Endpoint that service will connect to, tcp://ip:port")
        (@arg peers: --peer +takes_value +multiple
          "Endpoint that service will connect to, ip:port")
        (@arg ca_file: --("ca-file") +takes_value
          "File path to the trusted CA certificate")
        (@arg cert_dir: --("cert-dir") +takes_value
          "Path to the directory where the certificates and keys are")
        (@arg client_cert: --("client-cert") +takes_value
          "File path to the certificate for the node when connecting to a node")
        (@arg server_cert: --("server-cert") +takes_value
          "File path to the certificate for the node when connecting to a node")
        (@arg server_key:  --("server-key") +takes_value
          "File path to the key for the node when connecting to a node as server")
        (@arg client_key:  --("client-key") +takes_value
          "File path to the key for the node when connecting to a node as client")
        (@arg insecure:  --("insecure")
          "If set to tls, should accept all peer certificates")
        (@arg bind: --("bind") +takes_value
          "Connection endpoint for REST API")
        (@arg registry_backend: --("registry-backend") +takes_value
          "Backend type for the node registry. Possible values: FILE.")
        (@arg registry_file: --("registry-file") +takes_value
          "File path to the node registry file if registry-backend is FILE.")
        (@arg admin_service_coordinator_timeout: --("admin-timeout") +takes_value
            "The coordinator timeout for admin service proposals (in milliseconds); default is \
             30000 (30 seconds)")
        (@arg verbose: -v --verbose +multiple
          "Increase output verbosity"));

    let app = app.arg(
        Arg::with_name("heartbeat_interval")
            .long("heartbeat")
            .long_help(
                "How often heartbeat should be sent, in seconds; defaults to 30 seconds,\
                 0 means off",
            )
            .takes_value(true),
    );

    #[cfg(feature = "database")]
    let app = app.arg(
        Arg::with_name("database")
            .long("database")
            .long_help("DB connection URL")
            .takes_value(true),
    );

    #[cfg(feature = "biome")]
    let app = app.arg(
        Arg::with_name("biome_enabled")
            .long("enable-biome")
            .long_help("Enable the biome subsystem"),
    );

    #[cfg(feature = "generate-certs")]
    let app = app
        .arg(
            Arg::with_name("generate_certs")
                .long("generate-certs")
                .long_help(
                    "Deprecated: If set, certificates will be generated and insecure will be false; \
                     use only for development",
                ),
        )
        .arg(
            Arg::with_name("common_name")
                .long("common-name")
                .long_help(
                    "Deprecated: The common name that should be used in the generated certificate; \
                     defaults to localhost",
                )
                .takes_value(true),
        );

    let matches = app.get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    let mut log_spec_builder = LogSpecBuilder::new();
    log_spec_builder.default(log_level);
    log_spec_builder.module("hyper", log::LevelFilter::Warn);
    log_spec_builder.module("tokio", log::LevelFilter::Warn);

    Logger::with(log_spec_builder.build())
        .format(log_format)
        .start()
        .expect("Failed to create logger");

    if let Err(err) = start_daemon(matches) {
        error!("Failed to start daemon, {}", err);
        std::process::exit(1);
    }
}

fn start_daemon(matches: ArgMatches) -> Result<(), UserError> {
    debug!("Loading configuration file");

    // get provided config file or search default location
    let config_file = matches
        .value_of("config")
        .unwrap_or("/etc/splinter/splinterd.toml");

    let config_file_path = if Path::new(&config_file).is_file() {
        Some(config_file)
    } else {
        None
    };

    let final_config = create_config(config_file_path, matches.clone())?;

    let node_id = final_config.node_id();

    let storage_type = final_config.storage();

    let transport_type = final_config.transport();

    let service_endpoint = final_config.service_endpoint();

    let network_endpoint = final_config.network_endpoint();

    let initial_peers = final_config.peers();

    let heartbeat_interval = final_config.heartbeat_interval();

    let (transport, insecure) = get_transport(&transport_type, &matches, &final_config)?;

    let location = final_config.state_dir();

    let storage_location = match &storage_type as &str {
        "yaml" => format!("{}{}", location, "circuits.yaml"),
        "memory" => "memory".to_string(),
        _ => {
            return Err(UserError::InvalidArgument(format!(
                "storage type is not supported: {}",
                storage_type
            )))
        }
    };

    let key_registry_location = match &storage_type as &str {
        "yaml" => format!("{}{}", location, "keys.yaml"),
        "memory" => "memory".to_string(),
        _ => {
            return Err(UserError::InvalidArgument(format!(
                "storage type is not supported: {}",
                storage_type
            )))
        }
    };

    let rest_api_endpoint = final_config.bind();

    #[cfg(feature = "database")]
    let db_url = final_config.database();

    #[cfg(feature = "biome")]
    let biome_enabled: bool = matches.is_present("biome_enabled");

    let registry_backend = final_config.registry_backend();

    #[cfg(feature = "experimental")]
    // Allow unused mut for experimental features
    #[allow(unused_mut)]
    let mut feature_fields = "".to_string();

    let admin_service_coordinator_timeout = final_config.admin_service_coordinator_timeout();

    #[cfg(feature = "biome")]
    {
        debug!("{}, biome_enabled: {}", feature_fields, biome_enabled);
    }

    final_config.log_as_debug(insecure);

    let mut daemon_builder = SplinterDaemonBuilder::new()
        .with_storage_location(storage_location)
        .with_key_registry_location(key_registry_location)
        .with_network_endpoint(String::from(network_endpoint))
        .with_service_endpoint(String::from(service_endpoint))
        .with_initial_peers(initial_peers.to_vec())
        .with_node_id(String::from(node_id))
        .with_rest_api_endpoint(String::from(rest_api_endpoint))
        .with_storage_type(String::from(storage_type))
        .with_heartbeat_interval(heartbeat_interval)
        .with_admin_service_coordinator_timeout(admin_service_coordinator_timeout);

    #[cfg(feature = "database")]
    {
        daemon_builder = daemon_builder.with_db_url(Some(String::from(db_url)));
    }

    #[cfg(feature = "biome")]
    {
        daemon_builder = daemon_builder.enable_biome(biome_enabled);
    }

    if Path::new(&final_config.registry_file()).is_file() && registry_backend == "FILE" {
        daemon_builder = daemon_builder
            .with_registry_backend(Some(String::from(registry_backend)))
            .with_registry_file(String::from(final_config.registry_file()));
    } else {
        daemon_builder = daemon_builder.with_registry_backend(None);
    }

    let mut node = daemon_builder.build().map_err(|err| {
        UserError::daemon_err_with_source("unable to build the Splinter daemon", Box::new(err))
    })?;
    node.start(transport)?;
    Ok(())
}

fn get_transport(
    transport_type: &str,
    matches: &clap::ArgMatches,
    config: &Config,
) -> Result<(Box<dyn Transport + Send>, bool), GetTransportError> {
    match transport_type {
        "tls" => {
            #[cfg(feature = "generate-certs")]
            {
                if matches.is_present("generate_certs") {
                    warn!("Deprecated: Generating Certs for TLS Transport");

                    let common_name = matches
                        .value_of("common_name")
                        .map(String::from)
                        .unwrap_or_else(|| String::from("localhost"));

                    // Generate Certificate Authority keys and certificate
                    let (ca_key, ca_cert) = make_ca_cert()?;

                    // Create temp directory to store ca.cert
                    let temp_dir = TempDir::new("tls-transport")?;
                    let temp_dir_path = temp_dir.path();

                    // Generate client and server keys and certificates
                    let (client_key, client_cert) =
                        make_ca_signed_cert(&ca_cert, &ca_key, &common_name)?;
                    let (server_key, server_cert) =
                        make_ca_signed_cert(&ca_cert, &ca_key, &common_name)?;

                    let client_cert = write_file(
                        temp_dir_path.to_path_buf(),
                        "client.cert",
                        &client_cert.to_pem()?,
                    )?;

                    let client_key_file = write_file(
                        temp_dir_path.to_path_buf(),
                        "client.key",
                        &client_key.private_key_to_pem_pkcs8()?,
                    )?;

                    let server_cert = write_file(
                        temp_dir_path.to_path_buf(),
                        "server.cert",
                        &server_cert.to_pem()?,
                    )?;

                    let server_key_file = write_file(
                        temp_dir_path.to_path_buf(),
                        "server.key",
                        &server_key.private_key_to_pem_pkcs8()?,
                    )?;

                    // Start transport in insecure mode, do not verify the certs if auto generated,
                    // as the ca will not match
                    let transport = TlsTransport::new(
                        None,
                        client_key_file,
                        client_cert,
                        server_key_file,
                        server_cert,
                    )?;

                    return Ok((Box::new(transport), true));
                }
            }

            let client_cert = config.client_cert();
            if !Path::new(&client_cert).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid client certificate: {}",
                    client_cert
                )));
            }

            let server_cert = config.server_cert();
            if !Path::new(&server_cert).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid server certificate: {}",
                    server_cert
                )));
            }

            let server_key_file = config.server_key();
            if !Path::new(&server_key_file).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid server key path: {}",
                    server_key_file
                )));
            }

            let client_key_file = config.client_key();
            if !Path::new(&client_key_file).is_file() {
                return Err(GetTransportError::CertError(format!(
                    "Must provide a valid client key path: {}",
                    client_key_file
                )));
            }

            let insecure = matches.is_present("insecure");
            let ca_file = {
                if insecure {
                    None
                } else {
                    let ca_file = config.ca_certs();
                    if !Path::new(&ca_file).is_file() {
                        return Err(GetTransportError::CertError(format!(
                            "Must provide a valid file containing ca certs: {}",
                            ca_file
                        )));
                    }
                    match fs::canonicalize(&ca_file)?.to_str() {
                        Some(ca_path) => Some(ca_path.to_string()),
                        None => {
                            return Err(GetTransportError::CertError(
                                "CA path is not a valid path".to_string(),
                            ))
                        }
                    }
                }
            };

            let transport = TlsTransport::new(
                ca_file.map(String::from),
                String::from(client_key_file),
                String::from(client_cert),
                String::from(server_key_file),
                String::from(server_cert),
            )?;

            Ok((Box::new(transport), insecure))
        }
        "raw" => Ok((Box::new(RawTransport::default()), true)),
        _ => Err(GetTransportError::NotSupportedError(format!(
            "Transport type {} is not supported",
            transport_type
        ))),
    }
}

#[derive(Debug)]
pub enum UserError {
    TransportError(GetTransportError),
    MissingArgument(String),
    InvalidArgument(String),
    ConfigError(ConfigError),
    DaemonError {
        context: String,
        source: Option<Box<dyn Error>>,
    },
}

impl UserError {
    pub fn daemon_error(context: &str) -> Self {
        UserError::DaemonError {
            context: context.into(),
            source: None,
        }
    }

    pub fn daemon_err_with_source(context: &str, err: Box<dyn Error>) -> Self {
        UserError::DaemonError {
            context: context.into(),
            source: Some(err),
        }
    }
}

impl Error for UserError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            UserError::TransportError(err) => Some(err),
            UserError::MissingArgument(_) => None,
            UserError::InvalidArgument(_) => None,
            UserError::ConfigError(err) => Some(err),
            UserError::DaemonError { source, .. } => {
                if let Some(ref err) = source {
                    Some(&**err)
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UserError::TransportError(err) => write!(f, "unable to get transport: {}", err),
            UserError::MissingArgument(msg) => write!(f, "missing required argument: {}", msg),
            UserError::InvalidArgument(msg) => write!(f, "required argument is invalid: {}", msg),
            UserError::ConfigError(msg) => {
                write!(f, "error occurred building config object: {}", msg)
            }
            UserError::DaemonError { context, source } => {
                if let Some(ref err) = source {
                    write!(f, "{}: {}", context, err)
                } else {
                    f.write_str(&context)
                }
            }
        }
    }
}

impl From<StartError> for UserError {
    fn from(error: StartError) -> Self {
        UserError::daemon_err_with_source("unable to start the Splinter daemon", Box::new(error))
    }
}

impl From<GetTransportError> for UserError {
    fn from(error: GetTransportError) -> Self {
        UserError::TransportError(error)
    }
}

impl From<ConfigError> for UserError {
    fn from(error: ConfigError) -> Self {
        UserError::ConfigError(error)
    }
}

#[derive(Debug)]
pub enum GetTransportError {
    CertError(String),
    NotSupportedError(String),
    TlsTransportError(TlsInitError),
    #[cfg(feature = "generate-certs")]
    OpensslError(ErrorStack),
    IoError(io::Error),
}

impl Error for GetTransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GetTransportError::CertError(_) => None,
            GetTransportError::NotSupportedError(_) => None,
            GetTransportError::TlsTransportError(err) => Some(err),
            #[cfg(feature = "generate-certs")]
            GetTransportError::OpensslError(err) => Some(err),
            GetTransportError::IoError(err) => Some(err),
        }
    }
}

impl fmt::Display for GetTransportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GetTransportError::CertError(msg) => {
                write!(f, "unable to retrieve certificate: {}", msg)
            }
            GetTransportError::NotSupportedError(msg) => {
                write!(f, "received transport type that is not supported: {}", msg)
            }
            GetTransportError::TlsTransportError(err) => {
                write!(f, "unable to create TLS transport: {}", err)
            }
            #[cfg(feature = "generate-certs")]
            GetTransportError::OpensslError(err) => {
                write!(f, "unable to generate certificates: {}", err)
            }
            GetTransportError::IoError(err) => {
                write!(f, "unable to get transport due to IoError: {}", err)
            }
        }
    }
}

#[cfg(feature = "generate-certs")]
impl From<CertError> for GetTransportError {
    fn from(cert_error: CertError) -> Self {
        GetTransportError::CertError(format!("CertError: {:?}", cert_error))
    }
}

impl From<TlsInitError> for GetTransportError {
    fn from(tls_error: TlsInitError) -> Self {
        GetTransportError::TlsTransportError(tls_error)
    }
}

#[cfg(feature = "generate-certs")]
impl From<ErrorStack> for GetTransportError {
    fn from(error_stack: ErrorStack) -> Self {
        GetTransportError::OpensslError(error_stack)
    }
}

impl From<io::Error> for GetTransportError {
    fn from(io_error: io::Error) -> Self {
        GetTransportError::IoError(io_error)
    }
}
