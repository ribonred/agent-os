//! The one HTTPS client this shell uses to reach anything off the
//! device.
//!
//! Almost everything the shell talks to is the agent gateway on
//! loopback, in plaintext (see `agent::http_client`). The exceptions are
//! the handful of public services it calls directly, and they are all
//! https-only, so they share one client built one way.
//!
//! Roots are compiled in (webpki-roots) rather than read from the
//! device: the golden image is copied byte-for-byte onto every unit, and
//! depending on where a cert store happens to live is one more thing
//! that can differ between a build machine and a shipped unit.

use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;

pub type HttpsClient = Client<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>, Full<Bytes>>;

/// Built per call rather than held in a static. These calls are
/// occasional -- a catalogue refresh, a sentence to speak -- and a
/// connection pool that outlives them buys nothing worth the shared
/// state.
pub fn https_client() -> HttpsClient {
    let tls = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .build();
    Client::builder(TokioExecutor::new()).build(tls)
}
