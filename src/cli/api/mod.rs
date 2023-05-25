use std::io::Write;
use std::sync::Arc;

use base64::prelude::BASE64_STANDARD;
use base64::write::EncoderWriter;
use ureq::{Agent, Response};

use crate::api::tls::SkipServerVerification;
use crate::errors::AnyError;

pub mod node;
pub mod node_api;
pub mod stats;
mod tls;

pub struct Client {
    client: Agent,
    config: ClientConfig,
}

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
                &self.username.clone().unwrap_or(String::new())
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
        host: &str,
        port: &u16,
        username: Option<String>,
        password: Option<String>,
        skip_tls_verification: bool,
    ) -> Result<Self, AnyError> {
        let base_url = format!("{}:{}", host, port);
        let agent: Agent;

        if skip_tls_verification {
            let tls_config = rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(SkipServerVerification::new())
                .with_no_client_auth();

            agent = ureq::AgentBuilder::new()
                .tls_config(Arc::new(tls_config))
                .build();
        } else {
            agent = ureq::AgentBuilder::new().build()
        }

        Ok(Self {
            client: agent,
            config: ClientConfig {
                base_url,
                username,
                password,
            },
        })
    }

    pub fn request(
        &self,
        method: &str,
        request_path: &str,
        query: Option<&[(String, String)]>,
    ) -> Result<Response, AnyError> {
        let path = format!("{}/{}", self.config.base_url, request_path);
        let mut request = self.client.request(method, &path);

        if let Some(pairs) = query {
            for pair in pairs {
                request = request.query(&(*pair).0, &(*pair).1);
            }
        }

        if self.config.username.is_some() {
            request = request.set("authorization", &self.config.basic_auth_header())
        }

        let result = request.call()?;
        return Ok(result);
    }
}
