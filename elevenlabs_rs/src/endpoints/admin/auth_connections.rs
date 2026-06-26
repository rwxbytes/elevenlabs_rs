//! The workspace auth-connection endpoints.
//!
//! Auth connections store the credentials that tools and MCP servers use to
//! authenticate against upstream providers. Each connection has an
//! [`auth_type`](AuthConnectionConfig) that determines its configuration shape;
//! responses are modeled as a common envelope plus a type-specific
//! [`AuthConnectionConfig`].
//!
//! See the [Auth Connections API reference](https://elevenlabs.io/docs/api-reference/workspace/auth-connections).

use super::*;
use std::collections::HashMap;

// =============================================================================
// Shared types
// =============================================================================

/// The lifecycle status of an auth connection's stored credential.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthConnectionStatus {
    #[default]
    Active,
    RefreshFailed,
    Revoked,
    CredentialInvalid,
}

/// The signing algorithm used by JWT-based auth connections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum JwtAlgorithm {
    Hs256,
    Hs384,
    Hs512,
    Rs256,
    Rs384,
    Rs512,
}

/// Which field of an OAuth2 token response holds the token to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenResponseField {
    AccessToken,
    IdToken,
}

/// Dependencies (tools, MCP servers, integration connections) that use an auth
/// connection.
///
/// The identifiers are kept as raw JSON values: they are informational and use
/// several discriminated identifier shapes that are rarely needed by callers.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct AuthConnectionDependencies {
    #[serde(default)]
    pub tools: Vec<Value>,
    #[serde(default)]
    pub mcp_servers: Vec<Value>,
    #[serde(default)]
    pub integration_connections: Vec<Value>,
}

// =============================================================================
// Response model
// =============================================================================

/// A workspace auth connection.
///
/// The common envelope fields are typed directly; the credential
/// configuration, which varies by `auth_type`, is in [`config`](Self::config).
#[derive(Clone, Debug, Deserialize)]
pub struct AuthConnection {
    pub id: String,
    pub name: String,
    pub provider: Option<String>,
    pub used_by: Option<AuthConnectionDependencies>,
    #[serde(default)]
    pub status: AuthConnectionStatus,
    pub status_detail: Option<String>,
    pub status_updated_at: Option<String>,
    #[serde(flatten)]
    pub config: AuthConnectionConfig,
}

/// The type-specific configuration of an [`AuthConnection`], discriminated by
/// its `auth_type`. Secret material (passwords, client secrets, private keys)
/// is never returned, so it does not appear here.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum AuthConnectionConfig {
    Oauth2ClientCredentials {
        client_id: String,
        token_url: String,
        #[serde(default)]
        scopes: Vec<String>,
        #[serde(default)]
        extra_params: HashMap<String, String>,
        #[serde(default)]
        basic_auth_in_header: bool,
        #[serde(default)]
        custom_headers: HashMap<String, String>,
    },
    CustomHeaderAuth {
        header_name: String,
    },
    BasicAuth {
        username: String,
    },
    BearerAuth {},
    Oauth2Jwt {
        algorithm: Option<String>,
        key_id: Option<String>,
        issuer: String,
        audience: String,
        subject: String,
        expiration_seconds: Option<i64>,
        #[serde(default)]
        extra_params: HashMap<String, String>,
        token_url: String,
        #[serde(default)]
        scopes: Vec<String>,
        token_response_field: Option<String>,
    },
    PrivateKeyJwt {
        algorithm: Option<String>,
        key_id: Option<String>,
        issuer: String,
        audience: String,
        subject: String,
        expiration_seconds: Option<i64>,
        #[serde(default)]
        extra_params: HashMap<String, String>,
    },
    Mtls {},
    ApiIntegrationOauth2AuthCode {
        token_url: String,
        #[serde(default)]
        scopes: Vec<String>,
        scope_separator: Option<String>,
        expires_at: String,
        integration_id: String,
        credential_id: String,
    },
    ApiIntegrationOauth2CustomApp {
        client_id: String,
        token_url: String,
        #[serde(default)]
        scopes: Vec<String>,
        scope_separator: Option<String>,
        expires_at: String,
        integration_id: String,
        credential_id: String,
    },
    WhatsappAuth {
        phone_number_id: String,
    },
    SlackBotAuth {},
    UrlSecret {},
}

impl AuthConnection {
    /// The `auth_type` discriminator string for this connection.
    pub fn auth_type(&self) -> &'static str {
        match self.config {
            AuthConnectionConfig::Oauth2ClientCredentials { .. } => "oauth2_client_credentials",
            AuthConnectionConfig::CustomHeaderAuth { .. } => "custom_header_auth",
            AuthConnectionConfig::BasicAuth { .. } => "basic_auth",
            AuthConnectionConfig::BearerAuth {} => "bearer_auth",
            AuthConnectionConfig::Oauth2Jwt { .. } => "oauth2_jwt",
            AuthConnectionConfig::PrivateKeyJwt { .. } => "private_key_jwt",
            AuthConnectionConfig::Mtls {} => "mtls",
            AuthConnectionConfig::ApiIntegrationOauth2AuthCode { .. } => {
                "api_integration_oauth2_auth_code"
            }
            AuthConnectionConfig::ApiIntegrationOauth2CustomApp { .. } => {
                "api_integration_oauth2_custom_app"
            }
            AuthConnectionConfig::WhatsappAuth { .. } => "whatsapp_auth",
            AuthConnectionConfig::SlackBotAuth {} => "slack_bot_auth",
            AuthConnectionConfig::UrlSecret {} => "url_secret",
        }
    }
}

// =============================================================================
// POST /v1/workspace/auth-connections — Create Auth Connection
// =============================================================================

/// Creates a workspace auth connection from a typed request.
///
/// # Example
/// ```no_run
/// use elevenlabs_rs::{ElevenLabsClient, Result};
/// use elevenlabs_rs::endpoints::admin::auth_connections::{
///     CreateAuthConnection, CreateBearerAuth,
/// };
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let c = ElevenLabsClient::from_env()?;
///     let request = CreateBearerAuth::new("My API", "acme", "secret-token");
///     let connection = c.hit(CreateAuthConnection::new(request)).await?;
///     println!("{} ({})", connection.id, connection.auth_type());
///     Ok(())
/// }
/// ```
/// See [Create Auth Connection API reference](https://elevenlabs.io/docs/api-reference/workspace/create-auth-connection).
#[derive(Clone, Debug)]
pub struct CreateAuthConnection {
    body: CreateAuthConnectionRequest,
}

impl CreateAuthConnection {
    pub fn new(request: impl Into<CreateAuthConnectionRequest>) -> Self {
        Self {
            body: request.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for CreateAuthConnection {}

impl ElevenLabsEndpoint for CreateAuthConnection {
    const PATH: &'static str = "/v1/workspace/auth-connections";

    const METHOD: Method = Method::POST;

    type ResponseBody = AuthConnection;

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A typed request to create an auth connection. The `auth_type` discriminator
/// is added automatically during serialization.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum CreateAuthConnectionRequest {
    Oauth2ClientCredentials(CreateOAuth2ClientCreds),
    CustomHeaderAuth(CreateCustomHeaderAuth),
    BasicAuth(CreateBasicAuth),
    BearerAuth(CreateBearerAuth),
    Oauth2Jwt(CreateOAuth2Jwt),
    PrivateKeyJwt(CreatePrivateKeyJwt),
    Mtls(CreateMtls),
}

macro_rules! create_request_from {
    ($variant:ident, $ty:ident) => {
        impl From<$ty> for CreateAuthConnectionRequest {
            fn from(request: $ty) -> Self {
                CreateAuthConnectionRequest::$variant(request)
            }
        }
    };
}

create_request_from!(Oauth2ClientCredentials, CreateOAuth2ClientCreds);
create_request_from!(CustomHeaderAuth, CreateCustomHeaderAuth);
create_request_from!(BasicAuth, CreateBasicAuth);
create_request_from!(BearerAuth, CreateBearerAuth);
create_request_from!(Oauth2Jwt, CreateOAuth2Jwt);
create_request_from!(PrivateKeyJwt, CreatePrivateKeyJwt);
create_request_from!(Mtls, CreateMtls);

/// Create request for an OAuth2 client-credentials connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreateOAuth2ClientCreds {
    name: String,
    provider: String,
    client_id: String,
    client_secret: String,
    token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_params: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    basic_auth_in_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_headers: Option<HashMap<String, String>>,
}

impl CreateOAuth2ClientCreds {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            token_url: token_url.into(),
            scopes: None,
            extra_params: None,
            basic_auth_in_header: None,
            custom_headers: None,
        }
    }

    pub fn with_scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes = Some(scopes.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_extra_params(mut self, extra_params: HashMap<String, String>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }

    pub fn with_basic_auth_in_header(mut self, basic_auth_in_header: bool) -> Self {
        self.basic_auth_in_header = Some(basic_auth_in_header);
        self
    }

    pub fn with_custom_headers(mut self, custom_headers: HashMap<String, String>) -> Self {
        self.custom_headers = Some(custom_headers);
        self
    }
}

/// Create request for a custom-header auth connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreateCustomHeaderAuth {
    name: String,
    provider: String,
    header_name: String,
    token: String,
}

impl CreateCustomHeaderAuth {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        header_name: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            header_name: header_name.into(),
            token: token.into(),
        }
    }
}

/// Create request for a basic-auth connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreateBasicAuth {
    name: String,
    provider: String,
    username: String,
    password: String,
}

impl CreateBasicAuth {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            username: username.into(),
            password: password.into(),
        }
    }
}

/// Create request for a bearer-token auth connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreateBearerAuth {
    name: String,
    provider: String,
    token: String,
}

impl CreateBearerAuth {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            token: token.into(),
        }
    }
}

/// Create request for an OAuth2 JWT-bearer connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreateOAuth2Jwt {
    name: String,
    provider: String,
    issuer: String,
    audience: String,
    subject: String,
    secret_key: String,
    token_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    algorithm: Option<JwtAlgorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_params: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_response_field: Option<TokenResponseField>,
}

impl CreateOAuth2Jwt {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        issuer: impl Into<String>,
        audience: impl Into<String>,
        subject: impl Into<String>,
        secret_key: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            issuer: issuer.into(),
            audience: audience.into(),
            subject: subject.into(),
            secret_key: secret_key.into(),
            token_url: token_url.into(),
            algorithm: None,
            key_id: None,
            expiration_seconds: None,
            extra_params: None,
            scopes: None,
            token_response_field: None,
        }
    }

    pub fn with_algorithm(mut self, algorithm: JwtAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn with_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.key_id = Some(key_id.into());
        self
    }

    pub fn with_expiration_seconds(mut self, expiration_seconds: i64) -> Self {
        self.expiration_seconds = Some(expiration_seconds);
        self
    }

    pub fn with_extra_params(mut self, extra_params: HashMap<String, String>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }

    pub fn with_scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes = Some(scopes.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_token_response_field(mut self, token_response_field: TokenResponseField) -> Self {
        self.token_response_field = Some(token_response_field);
        self
    }
}

/// Create request for a private-key JWT connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreatePrivateKeyJwt {
    name: String,
    provider: String,
    issuer: String,
    audience: String,
    subject: String,
    secret_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    algorithm: Option<JwtAlgorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_params: Option<HashMap<String, String>>,
}

impl CreatePrivateKeyJwt {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        issuer: impl Into<String>,
        audience: impl Into<String>,
        subject: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            issuer: issuer.into(),
            audience: audience.into(),
            subject: subject.into(),
            secret_key: secret_key.into(),
            algorithm: None,
            key_id: None,
            expiration_seconds: None,
            extra_params: None,
        }
    }

    pub fn with_algorithm(mut self, algorithm: JwtAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn with_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.key_id = Some(key_id.into());
        self
    }

    pub fn with_expiration_seconds(mut self, expiration_seconds: i64) -> Self {
        self.expiration_seconds = Some(expiration_seconds);
        self
    }

    pub fn with_extra_params(mut self, extra_params: HashMap<String, String>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }
}

/// Create request for a mutual-TLS connection.
#[derive(Clone, Debug, Serialize)]
pub struct CreateMtls {
    name: String,
    provider: String,
    client_certificate: String,
    client_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ca_certificate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_passphrase: Option<String>,
}

impl CreateMtls {
    pub fn new(
        name: impl Into<String>,
        provider: impl Into<String>,
        client_certificate: impl Into<String>,
        client_key: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            client_certificate: client_certificate.into(),
            client_key: client_key.into(),
            ca_certificate: None,
            key_passphrase: None,
        }
    }

    pub fn with_ca_certificate(mut self, ca_certificate: impl Into<String>) -> Self {
        self.ca_certificate = Some(ca_certificate.into());
        self
    }

    pub fn with_key_passphrase(mut self, key_passphrase: impl Into<String>) -> Self {
        self.key_passphrase = Some(key_passphrase.into());
        self
    }
}

// =============================================================================
// GET /v1/workspace/auth-connections — List Auth Connections
// =============================================================================

/// Lists the workspace's auth connections.
///
/// See [List Auth Connections API reference](https://elevenlabs.io/docs/api-reference/workspace/list-auth-connections).
#[derive(Clone, Debug, Default)]
pub struct ListAuthConnections;

impl crate::endpoints::sealed::Sealed for ListAuthConnections {}

impl ElevenLabsEndpoint for ListAuthConnections {
    const PATH: &'static str = "/v1/workspace/auth-connections";

    const METHOD: Method = Method::GET;

    type ResponseBody = ListAuthConnectionsResponse;

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// The response of [`ListAuthConnections`].
#[derive(Clone, Debug, Deserialize)]
pub struct ListAuthConnectionsResponse {
    pub auth_connections: Vec<AuthConnection>,
}

impl IntoIterator for ListAuthConnectionsResponse {
    type Item = AuthConnection;
    type IntoIter = std::vec::IntoIter<AuthConnection>;

    fn into_iter(self) -> Self::IntoIter {
        self.auth_connections.into_iter()
    }
}

// =============================================================================
// PATCH /v1/workspace/auth-connections/{auth_connection_id} — Update
// =============================================================================

/// Updates a workspace auth connection. Only client-credentials, basic, bearer,
/// and OAuth2-JWT connections can be updated.
///
/// See [Update Auth Connection API reference](https://elevenlabs.io/docs/api-reference/workspace/update-auth-connection).
#[derive(Clone, Debug)]
pub struct UpdateAuthConnection {
    auth_connection_id: String,
    body: UpdateAuthConnectionRequest,
}

impl UpdateAuthConnection {
    pub fn new(
        auth_connection_id: impl Into<String>,
        request: impl Into<UpdateAuthConnectionRequest>,
    ) -> Self {
        Self {
            auth_connection_id: auth_connection_id.into(),
            body: request.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for UpdateAuthConnection {}

impl ElevenLabsEndpoint for UpdateAuthConnection {
    const PATH: &'static str = "/v1/workspace/auth-connections/:auth_connection_id";

    const METHOD: Method = Method::PATCH;

    type ResponseBody = AuthConnection;

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .auth_connection_id
            .and_param(PathParam::AuthConnectionID)]
    }

    async fn request_body(&self) -> Result<RequestBody> {
        Ok(RequestBody::Json(serde_json::to_value(&self.body)?))
    }

    async fn response_body(self, resp: Response) -> Result<Self::ResponseBody> {
        Ok(resp.json().await?)
    }
}

/// A typed request to update an auth connection. The `auth_type` discriminator
/// is added automatically during serialization. All fields are optional.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "auth_type", rename_all = "snake_case")]
pub enum UpdateAuthConnectionRequest {
    Oauth2ClientCredentials(UpdateOAuth2ClientCreds),
    BasicAuth(UpdateBasicAuth),
    BearerAuth(UpdateBearerAuth),
    Oauth2Jwt(UpdateOAuth2Jwt),
}

impl From<UpdateOAuth2ClientCreds> for UpdateAuthConnectionRequest {
    fn from(request: UpdateOAuth2ClientCreds) -> Self {
        Self::Oauth2ClientCredentials(request)
    }
}

impl From<UpdateBasicAuth> for UpdateAuthConnectionRequest {
    fn from(request: UpdateBasicAuth) -> Self {
        Self::BasicAuth(request)
    }
}

impl From<UpdateBearerAuth> for UpdateAuthConnectionRequest {
    fn from(request: UpdateBearerAuth) -> Self {
        Self::BearerAuth(request)
    }
}

impl From<UpdateOAuth2Jwt> for UpdateAuthConnectionRequest {
    fn from(request: UpdateOAuth2Jwt) -> Self {
        Self::Oauth2Jwt(request)
    }
}

/// Update request for an OAuth2 client-credentials connection.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateOAuth2ClientCreds {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_params: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    basic_auth_in_header: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_headers: Option<HashMap<String, String>>,
}

impl UpdateOAuth2ClientCreds {
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn with_scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes = Some(scopes.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_extra_params(mut self, extra_params: HashMap<String, String>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }

    pub fn with_basic_auth_in_header(mut self, basic_auth_in_header: bool) -> Self {
        self.basic_auth_in_header = Some(basic_auth_in_header);
        self
    }

    pub fn with_custom_headers(mut self, custom_headers: HashMap<String, String>) -> Self {
        self.custom_headers = Some(custom_headers);
        self
    }
}

/// Update request for a basic-auth connection.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateBasicAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
}

impl UpdateBasicAuth {
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }
}

/// Update request for a bearer-token connection.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateBearerAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

impl UpdateBearerAuth {
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }
}

/// Update request for an OAuth2 JWT-bearer connection.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateOAuth2Jwt {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    algorithm: Option<JwtAlgorithm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audience: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_params: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_response_field: Option<TokenResponseField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_key: Option<String>,
}

impl UpdateOAuth2Jwt {
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    pub fn with_algorithm(mut self, algorithm: JwtAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    pub fn with_key_id(mut self, key_id: impl Into<String>) -> Self {
        self.key_id = Some(key_id.into());
        self
    }

    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn with_expiration_seconds(mut self, expiration_seconds: i64) -> Self {
        self.expiration_seconds = Some(expiration_seconds);
        self
    }

    pub fn with_extra_params(mut self, extra_params: HashMap<String, String>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }

    pub fn with_scopes<I, S>(mut self, scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.scopes = Some(scopes.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_token_response_field(mut self, token_response_field: TokenResponseField) -> Self {
        self.token_response_field = Some(token_response_field);
        self
    }

    pub fn with_secret_key(mut self, secret_key: impl Into<String>) -> Self {
        self.secret_key = Some(secret_key.into());
        self
    }
}

// =============================================================================
// DELETE /v1/workspace/auth-connections/{auth_connection_id} — Delete
// =============================================================================

/// Deletes a workspace auth connection.
///
/// See [Delete Auth Connection API reference](https://elevenlabs.io/docs/api-reference/workspace/delete-auth-connection).
#[derive(Clone, Debug)]
pub struct DeleteAuthConnection {
    auth_connection_id: String,
}

impl DeleteAuthConnection {
    pub fn new(auth_connection_id: impl Into<String>) -> Self {
        Self {
            auth_connection_id: auth_connection_id.into(),
        }
    }
}

impl crate::endpoints::sealed::Sealed for DeleteAuthConnection {}

impl ElevenLabsEndpoint for DeleteAuthConnection {
    const PATH: &'static str = "/v1/workspace/auth-connections/:auth_connection_id";

    const METHOD: Method = Method::DELETE;

    type ResponseBody = ();

    fn path_params(&self) -> Vec<(&'static str, &str)> {
        vec![self
            .auth_connection_id
            .and_param(PathParam::AuthConnectionID)]
    }

    async fn response_body(self, _resp: Response) -> Result<Self::ResponseBody> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_request_serializes_with_auth_type_discriminator() {
        let request = CreateAuthConnectionRequest::from(
            CreateOAuth2ClientCreds::new("My API", "acme", "client", "secret", "https://t.example")
                .with_scopes(["read", "write"]),
        );
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({
                "auth_type": "oauth2_client_credentials",
                "name": "My API",
                "provider": "acme",
                "client_id": "client",
                "client_secret": "secret",
                "token_url": "https://t.example",
                "scopes": ["read", "write"],
            })
        );
    }

    #[test]
    fn update_request_serializes_only_set_fields() {
        let request =
            UpdateAuthConnectionRequest::from(UpdateBearerAuth::default().with_token("new-token"));
        assert_eq!(
            serde_json::to_value(&request).unwrap(),
            json!({ "auth_type": "bearer_auth", "token": "new-token" })
        );
    }

    #[test]
    fn jwt_algorithm_serializes_uppercase() {
        assert_eq!(
            serde_json::to_value(JwtAlgorithm::Rs512).unwrap(),
            json!("RS512")
        );
    }

    #[test]
    fn response_deserializes_into_typed_config() {
        let connection: AuthConnection = serde_json::from_value(json!({
            "id": "ac_1",
            "name": "My Bearer",
            "provider": "acme",
            "auth_type": "bearer_auth",
            "status": "active",
        }))
        .unwrap();
        assert_eq!(connection.auth_type(), "bearer_auth");
        assert!(matches!(
            connection.config,
            AuthConnectionConfig::BearerAuth {}
        ));
        assert_eq!(connection.status, AuthConnectionStatus::Active);

        let connection: AuthConnection = serde_json::from_value(json!({
            "id": "ac_2",
            "name": "My OAuth",
            "provider": "acme",
            "auth_type": "oauth2_client_credentials",
            "client_id": "client",
            "token_url": "https://t.example",
            "scopes": ["read"],
            "status": "revoked",
        }))
        .unwrap();
        assert_eq!(connection.status, AuthConnectionStatus::Revoked);
        match connection.config {
            AuthConnectionConfig::Oauth2ClientCredentials {
                client_id, scopes, ..
            } => {
                assert_eq!(client_id, "client");
                assert_eq!(scopes, vec!["read".to_string()]);
            }
            other => panic!("unexpected config: {other:?}"),
        }
    }
}
