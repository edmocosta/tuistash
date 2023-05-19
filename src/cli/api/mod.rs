use std::io::Write;

use base64::prelude::BASE64_STANDARD;
use base64::write::EncoderWriter;
use ureq::Response;

use crate::errors::AnyError;

mod tls;
pub mod node_api;
pub mod stats;
pub mod node;

pub struct Client {
    client: ureq::Agent,
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
            let _ = write!(encoder, "{}:", &self.username.clone().unwrap_or(String::new()));
            if let Some(password) = &self.password {
                let _ = write!(encoder, "{}", password);
            }
        }

        String::from_utf8(buf).unwrap()
    }
}

impl Client {
    pub fn new(host: &str, port: &u16, username: Option<String>, password: Option<String>) -> Result<Self, AnyError> {
        let base_url = format!("{}:{}", host, port);

        // let tls_config = rustls::ClientConfig::builder()
        //     .with_safe_defaults()
        //     .with_custom_certificate_verifier(SkipServerVerification::new())
        //     .with_no_client_auth();

        Ok(Self {
            client: ureq::AgentBuilder::new().build(),
            config: ClientConfig { base_url, username, password },
        })
    }

    pub fn request(&self, method: &str, request_path: &str, query: Option<&[(String, String)]>) -> Result<Response, AnyError> {
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

