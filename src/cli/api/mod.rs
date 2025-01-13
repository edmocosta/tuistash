use std::io::Write;
use std::sync::Arc;

use base64::prelude::BASE64_STANDARD;
use base64::write::EncoderWriter;
use ureq::{Agent, AgentBuilder, Response};

use crate::api::tls::SkipServerVerification;
use crate::errors::AnyError;

pub mod hot_threads;
pub mod node;
pub mod node_api;
pub mod stats;
mod tls;

#[derive(Debug, Clone)]
pub struct Client {
    client: Agent,
    config: ClientConfig,
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    base_url: String,
    username: Option<String>,
    password: Option<String>,
}

impl ClientConfig {
    pub fn basic_auth_header(&self) -> String {
        let mut buf = b"Basic ".to_vec();
        {
            let mut encoder = EncoderWriter::new(&mut buf, &BASE64_STANDARD);
            let _ = write!(
                encoder,
                "{}:",
                &self.username.as_ref().unwrap_or(&"".to_string())
            );
            if let Some(password) = &self.password {
                let _ = write!(encoder, "{}", password);
            }
        }

        String::from_utf8(buf).unwrap()
    }
}

impl Client {
    pub fn new(
        host: String,
        username: Option<String>,
        password: Option<String>,
        skip_tls_verification: bool,
    ) -> Result<Self, AnyError> {
        let user_agent = "tuistash";

        let agent_builder: AgentBuilder = if skip_tls_verification {
            let tls_config = rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(SkipServerVerification::new())
                .with_no_client_auth();
            AgentBuilder::new()
                .user_agent(user_agent)
                .tls_config(Arc::new(tls_config))
        } else {
            AgentBuilder::new().user_agent(user_agent)
        }
        .user_agent(format!("tuistash/{}", env!("CARGO_PKG_VERSION")).as_str());

        Ok(Self {
            client: agent_builder.build(),
            config: ClientConfig {
                base_url: host.to_string(),
                username,
                password,
            },
        })
    }

    pub fn request(
        &self,
        method: &str,
        request_path: &str,
        query: Option<&[(&str, &str)]>,
    ) -> Result<Response, AnyError> {
        let path = format!("{}/{}", self.config.base_url, request_path);
        let mut request = self.client.request(method, &path);

        if let Some(pairs) = query {
            for pair in pairs {
                request = request.query(pair.0, pair.1);
            }
        }

        if self.config.username.is_some() {
            request = request.set("authorization", &self.config.basic_auth_header())
        }

        let result = request.call()?;
        Ok(result)
    }

    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }
}
