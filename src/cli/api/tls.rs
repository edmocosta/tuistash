// use std::sync::Arc;
//
// struct SkipServerVerification;
//
// impl SkipServerVerification {
//     fn new() -> Arc<Self> {
//         Arc::new(Self)
//     }
// }
//
// impl rustls::client::ServerCertVerifier for SkipServerVerification {
//     fn verify_server_cert(
//         &self,
//         _end_entity: &rustls::Certificate,
//         _intermediates: &[rustls::Certificate],
//         _server_name: &rustls::ServerName,
//         _scts: &mut dyn Iterator<Item=&[u8]>,
//         _ocsp_response: &[u8],
//         _now: std::time::SystemTime,
//     ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
//         Ok(rustls::client::ServerCertVerified::assertion())
//     }
// }