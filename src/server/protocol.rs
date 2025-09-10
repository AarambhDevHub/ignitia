//! Protocol-detection utilities used by the Ignitia HTTP server.
//!
//! The server can operate in three modes:
//!   • HTTP/1.1 only
//!   • HTTP/2 only
//!   • Automatic detection (decide per connection)
//!
//! When TLS is enabled, ALPN (Application-Layer Protocol Negotiation) is
//! leveraged to discover whether the client prefers `h2` (HTTP/2) or
//! `http/1.1`.
//!
//! Without TLS—or when ALPN is unavailable—we fall back to the configuration
//! flags in `ServerConfig`:
//!   • `auto_protocol_detection = true`  ➜ probe each connection.
//!   • `http2.enable_prior_knowledge = true` ➜ treat clear-text traffic as
//!     HTTP/2 if the first client bytes match the HTTP/2 connection preface
//!     (handled elsewhere).
//!
//! This file keeps the surface area minimal so it can be reused by both the
//! plain-text and TLS connection paths with no extra dependencies.

/// Enumerates the protocols the server knows how to serve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpProtocol {
    /// HTTP/1.0 or HTTP/1.1
    Http1,
    /// HTTP/2 (h2 / h2c)
    Http2,
    /// “I don’t know yet – decide later”.
    ///
    /// When returned by a detector, it means:
    ///   • No definitive signal was found, or
    ///   • The server was built **without** TLS support, so ALPN is
    ///     unavailable.
    /// In this state the caller will look at its own run-time flags to pick a
    /// default or to attempt an in-band detection (e.g., by peeking at the
    /// connection preface).
    Auto,
}

/// Helper object; currently stateless but separated to isolate feature-gated
/// code behind `cfg(feature = "tls")`.
///
/// Additional detection strategies (e.g., Prior-Knowledge preface sniffing)
/// could be added here in the future without touching call-sites.
pub struct ProtocolDetector;

impl ProtocolDetector {
    /// Inspect the ALPN value produced by the TLS handshake and translate it
    /// into an `HttpProtocol`.
    ///
    /// *If the binary is built without the `tls` feature* this function is
    /// compiled to a stub that always returns `HttpProtocol::Auto`, allowing
    /// non-TLS builds to link without conditional logic in the caller.
    #[cfg(feature = "tls")]
    pub fn detect_from_alpn(alpn_protocol: Option<&[u8]>) -> HttpProtocol {
        match alpn_protocol {
            // The official ALPN identifier for HTTP/2 is exactly “h2”.
            Some(b"h2") => HttpProtocol::Http2,
            // Both HTTP/1.1 and HTTP/1.0 are mapped to the same branch because
            // the hyper back-end treats them uniformly once negotiated.
            Some(b"http/1.1") | Some(b"http/1.0") => HttpProtocol::Http1,
            // Unknown or absent ALPN => defer the choice.
            _ => HttpProtocol::Auto,
        }
    }

    /// Stub when TLS is disabled – keeps the API stable.
    #[cfg(not(feature = "tls"))]
    #[inline]
    pub fn detect_from_alpn(_: Option<&[u8]>) -> HttpProtocol {
        HttpProtocol::Auto
    }
}
