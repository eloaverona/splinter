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

use std::sync::Arc;

use crate::actix_web::HttpResponse;
use crate::futures::{Future, IntoFuture};
use crate::rest_api::{get_authorization_token, into_bytes, ErrorResponse, Method, Resource};

use super::super::rest_api::BiomeRestConfig;
use super::super::secrets::SecretManager;
use super::super::sessions::{validate_token, Claims, TokenValidationError};

use super::{store::KeyStore, Key};

#[derive(Deserialize)]
struct NewKey {
    public_key: String,
    encrypted_private_key: String,
    display_name: String,
}

/// Defines a REST endpoint for login
pub fn make_key_management_route(
    // credentials_store: Arc<SplinterCredentialsStore>,
    rest_config: Arc<BiomeRestConfig>,
    key_store: Arc<dyn KeyStore<Key>>,
    secret_manager: Arc<dyn SecretManager>,
) -> Resource {
    Resource::build("/biome/users/{user_id}/keys").add_method(
        Method::Post,
        move |request, payload| {
            // let credentials_store = credentials_store.clone();
            let rest_config = rest_config.clone();
            let key_store = key_store.clone();
            // let token_issuer = token_issuer.clone();
            let secret_manager = secret_manager.clone();
            let auth_token = match get_authorization_token(&request) {
                Ok(token) => token,
                Err(err) => {
                    debug!("Failed to get token: {}", err);
                    return Box::new(
                        HttpResponse::Unauthorized()
                            .json(ErrorResponse::unauthorized("User is not authorized"))
                            .into_future(),
                    );
                }
            };
            let user_id = request
                .match_info()
                .get("user_id")
                .unwrap_or_default()
                .to_owned();
            if let Err(err) = validate_token(
                &auth_token,
                &secret_manager.secret().unwrap(),
                &rest_config.issuer(),
                |claim| {
                    if user_id != claim.user_id() {
                        return Err(TokenValidationError::InvalidClaim(format!(
                            "User is not authorized to add keys for user {}",
                            user_id
                        )));
                    }
                    Ok(())
                },
            ) {
                debug!("Invalid token: {}", err);
                return Box::new(
                    HttpResponse::Unauthorized()
                        .json(ErrorResponse::unauthorized("User is not authorized"))
                        .into_future(),
                );
            };

            Box::new(into_bytes(payload).and_then(move |bytes| {
                let new_key = match serde_json::from_slice::<NewKey>(&bytes) {
                    Ok(val) => val,
                    Err(err) => {
                        debug!("Error parsing payload {}", err);
                        return HttpResponse::BadRequest()
                            .json(ErrorResponse::bad_request(&format!(
                                "Failed to parse payload: {}",
                                err
                            )))
                            .into_future();
                    }
                };
                let key = Key::new(
                    &new_key.public_key,
                    &new_key.encrypted_private_key,
                    &user_id,
                    &new_key.display_name,
                );

                match key_store.add_key(key) {
                    Ok(()) => HttpResponse::Ok()
                        .json(json!({ "message": "Key added successfully" }))
                        .into_future(),
                    Err(err) => {
                        debug!("Failed to add new key to database {}", err);
                        HttpResponse::InternalServerError()
                            .json(ErrorResponse::internal_error())
                            .into_future()
                    }
                }
            }))
        },
    )
    //     Box::new(HttpResponse::Ok().json(format!("auto {:?}", auth_token)).into_future())
    // })
}
