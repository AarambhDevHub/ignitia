//! TLS (Transport Layer Security) configuration and utilities for the Ignitia web framework.
//!
//! This module provides comprehensive TLS support including:
//! - Certificate and private key loading from PEM files
//! - ALPN (Application-Layer Protocol Negotiation) configuration for HTTP/2 and HTTP/1.1
//! - Self-signed certificate generation for development
//! - TLS version control and security settings
//! - Integration with tokio-rustls for async TLS operations
//!
//! # Features
//!
//! This module is only available when the `tls` feature is enabled. For self-signed certificate
//! generation, the `self-signed` feature must also be enabled.
//!
//! # Usage
//!
//! ## Basic TLS Setup
//!
//! ```
//! use ignitia::{Router, Server, TlsConfig};
//! use std::net::SocketAddr;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let router = Router::new()
//!     .get("/", || async { Ok(ignitia::Response::text("Hello HTTPS!")) });
//!
//! let tls_config = TlsConfig::new("cert.pem", "key.pem");
//! let addr: SocketAddr = "127.0.0.1:8443".parse()?;
//!
//! Server::new(router, addr)
//!     .with_tls(tls_config)?
//!     .ignitia()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Advanced Configuration
//!
//! ```
//! use ignitia::{TlsConfig, TlsVersion};
//!
//! let tls_config = TlsConfig::new("cert.pem", "key.pem")
//!     .with_alpn_protocols(vec!["h2", "http/1.1"])
//!     .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
//!     .enable_client_cert_verification();
//! ```
//!
//! ## Development with Self-Signed Certificates
//!
//! ```
//! # #[cfg(feature = "self-signed")]
//! use ignitia::TlsConfig;
//!
//! # #[cfg(feature = "self-signed")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Generate self-signed certificate for localhost
//! let (cert_pem, key_pem) = TlsConfig::generate_self_signed("localhost")?;
//! let tls_config = TlsConfig::new("self_signed_cert.pem", "self_signed_key.pem");
//! # Ok(())
//! # }
//! ```
//!
//! # Security Considerations
//!
//! - Always use strong certificates from trusted CAs in production
//! - Self-signed certificates should only be used in development
//! - Consider enabling client certificate verification for enhanced security
//! - Regularly update TLS versions to disable deprecated protocols
//! - Monitor certificate expiration dates

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use rustls::{Certificate, PrivateKey, ServerConfig};
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use rustls_pemfile::{certs, pkcs8_private_keys};
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use std::fs::File;
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use std::io::{self, BufReader, ErrorKind};
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use std::path::Path;
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use std::sync::Arc;
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use tokio_rustls::TlsAcceptor;
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
use tracing::{info, warn};

/// TLS configuration for HTTPS servers.
///
/// This struct encapsulates all TLS-related settings including certificate paths,
/// ALPN protocols, client authentication requirements, and supported TLS versions.
///
/// # Examples
///
/// ```
/// use ignitia::{TlsConfig, TlsVersion};
///
/// // Basic configuration
/// let config = TlsConfig::new("server.crt", "server.key");
///
/// // Advanced configuration
/// let config = TlsConfig::new("server.crt", "server.key")
///     .with_alpn_protocols(vec!["h2", "http/1.1", "http/1.0"])
///     .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13)
///     .enable_client_cert_verification();
/// ```
///
/// # Security Notes
///
/// - Certificate and key files must be in PEM format
/// - Private keys should be protected with appropriate file permissions (600)
/// - Consider using hardware security modules for private key storage in production
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to the server certificate file in PEM format.
    ///
    /// The certificate file should contain the server's public certificate,
    /// optionally followed by intermediate certificates in the certificate chain.
    pub cert_file: String,

    /// Path to the server's private key file in PEM format.
    ///
    /// The private key must correspond to the public key in the certificate.
    /// Supported formats include PKCS#8 and traditional RSA/EC private keys.
    pub key_file: String,

    /// ALPN (Application-Layer Protocol Negotiation) protocols to advertise.
    ///
    /// This list determines which protocols the server supports and their priority order.
    /// Common values:
    /// - `b"h2"` for HTTP/2
    /// - `b"http/1.1"` for HTTP/1.1
    /// - `b"http/1.0"` for HTTP/1.0
    ///
    /// The client will select the first protocol it supports from this list.
    pub alpn_protocols: Vec<Vec<u8>>,

    /// Whether to require and verify client certificates.
    ///
    /// When enabled, clients must present a valid certificate signed by a trusted CA.
    /// This provides mutual TLS authentication for enhanced security.
    pub client_cert_verification: bool,

    /// Minimum supported TLS version.
    ///
    /// Connections using older TLS versions will be rejected.
    /// For security, consider using TLS 1.2 or higher.
    pub min_tls_version: TlsVersion,

    /// Maximum supported TLS version.
    ///
    /// This allows limiting the TLS version for compatibility reasons,
    /// though normally you should use the latest available version.
    pub max_tls_version: TlsVersion,
}

/// Supported TLS protocol versions.
///
/// This enum represents the TLS versions that the server can negotiate with clients.
/// Each version provides different security and performance characteristics.
///
/// # Security Recommendations
///
/// - **TLS 1.3**: Latest version with improved security and performance
/// - **TLS 1.2**: Widely supported, secure when properly configured
/// - **TLS 1.1 and below**: Deprecated and should be avoided
///
/// # Examples
///
/// ```
/// use ignitia::{TlsConfig, TlsVersion};
///
/// let config = TlsConfig::new("cert.pem", "key.pem")
///     .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13);
/// ```
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
#[derive(Debug, Clone)]
pub enum TlsVersion {
    /// TLS version 1.2
    ///
    /// Widely supported version that provides strong security when properly configured.
    /// Required by many compliance standards and supported by virtually all modern clients.
    TlsV12,

    /// TLS version 1.3
    ///
    /// Latest TLS version with improved security, reduced handshake latency,
    /// and better cipher suite selection. Recommended for new deployments.
    TlsV13,
}

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
impl Default for TlsConfig {
    /// Creates a default TLS configuration.
    ///
    /// Default settings:
    /// - Certificate file: "cert.pem"
    /// - Private key file: "key.pem"
    /// - ALPN protocols: ["h2", "http/1.1"] (HTTP/2 preferred)
    /// - Client certificate verification: disabled
    /// - TLS versions: 1.2 to 1.3
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::TlsConfig;
    ///
    /// let config = TlsConfig::default();
    /// // Equivalent to:
    /// let config = TlsConfig::new("cert.pem", "key.pem");
    /// ```
    fn default() -> Self {
        Self {
            cert_file: "cert.pem".to_string(),
            key_file: "key.pem".to_string(),
            alpn_protocols: vec![
                b"h2".to_vec(),       // HTTP/2
                b"http/1.1".to_vec(), // HTTP/1.1
            ],
            client_cert_verification: false,
            min_tls_version: TlsVersion::TlsV12,
            max_tls_version: TlsVersion::TlsV13,
        }
    }
}

#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
impl TlsConfig {
    /// Creates a new TLS configuration with the specified certificate and key files.
    ///
    /// This constructor initializes a TLS configuration with sensible defaults:
    /// - ALPN protocols: HTTP/2 and HTTP/1.1
    /// - TLS versions: 1.2 and 1.3
    /// - Client certificate verification: disabled
    ///
    /// # Arguments
    ///
    /// * `cert_file` - Path to the PEM-encoded certificate file
    /// * `key_file` - Path to the PEM-encoded private key file
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::TlsConfig;
    ///
    /// let config = TlsConfig::new("server.crt", "server.key");
    /// let config = TlsConfig::new("/etc/ssl/certs/server.pem", "/etc/ssl/private/server.key");
    /// ```
    ///
    /// # Security Notes
    ///
    /// - Ensure certificate and key files are readable by the server process
    /// - Private key files should have restrictive permissions (e.g., 600)
    /// - Certificate files should contain the complete certificate chain
    pub fn new(cert_file: impl Into<String>, key_file: impl Into<String>) -> Self {
        Self {
            cert_file: cert_file.into(),
            key_file: key_file.into(),
            ..Default::default()
        }
    }

    /// Sets the ALPN (Application-Layer Protocol Negotiation) protocols.
    ///
    /// ALPN allows the server to advertise supported protocols during the TLS handshake,
    /// enabling clients to select the optimal protocol (e.g., HTTP/2 vs HTTP/1.1).
    ///
    /// # Arguments
    ///
    /// * `protocols` - Vector of protocol identifiers in preference order
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::TlsConfig;
    ///
    /// // Prefer HTTP/2, fallback to HTTP/1.1
    /// let config = TlsConfig::new("cert.pem", "key.pem")
    ///     .with_alpn_protocols(vec!["h2", "http/1.1"]);
    ///
    /// // HTTP/1.1 only
    /// let config = TlsConfig::new("cert.pem", "key.pem")
    ///     .with_alpn_protocols(vec!["http/1.1"]);
    /// ```
    ///
    /// # Common Protocol Identifiers
    ///
    /// - `"h2"` - HTTP/2 over TLS
    /// - `"http/1.1"` - HTTP/1.1
    /// - `"http/1.0"` - HTTP/1.0 (rarely used)
    ///
    /// # Performance Notes
    ///
    /// HTTP/2 generally provides better performance due to multiplexing and header compression,
    /// so it should typically be listed first when supported.
    pub fn with_alpn_protocols(mut self, protocols: Vec<&str>) -> Self {
        self.alpn_protocols = protocols
            .into_iter()
            .map(|p| p.as_bytes().to_vec())
            .collect();
        self
    }

    /// Enables client certificate verification for mutual TLS authentication.
    ///
    /// When enabled, clients must present a valid certificate during the TLS handshake.
    /// The server will verify the client certificate against configured trust anchors.
    ///
    /// # Security Benefits
    ///
    /// - Provides strong authentication of client identity
    /// - Prevents unauthorized access at the TLS layer
    /// - Enables non-repudiation through certificate-based identity
    ///
    /// # Requirements
    ///
    /// - Client certificates must be signed by a trusted CA
    /// - Clients must have access to their private keys
    /// - Certificate revocation checking may be required
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::TlsConfig;
    ///
    /// let config = TlsConfig::new("server.crt", "server.key")
    ///     .enable_client_cert_verification();
    /// ```
    ///
    /// # Note
    ///
    /// This is a basic implementation. Production deployments may require additional
    /// configuration for CA certificates and certificate revocation checking.
    pub fn enable_client_cert_verification(mut self) -> Self {
        self.client_cert_verification = true;
        self
    }

    /// Sets the supported TLS version range.
    ///
    /// This method configures the minimum and maximum TLS versions that the server
    /// will negotiate with clients. Connections requesting unsupported versions
    /// will be rejected.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum TLS version to accept
    /// * `max` - Maximum TLS version to support
    ///
    /// # Security Considerations
    ///
    /// - **TLS 1.3**: Recommended for new deployments (better security and performance)
    /// - **TLS 1.2**: Minimum recommended version for production use
    /// - **TLS 1.1 and below**: Deprecated and should be avoided
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::{TlsConfig, TlsVersion};
    ///
    /// // Only TLS 1.3
    /// let config = TlsConfig::new("cert.pem", "key.pem")
    ///     .tls_versions(TlsVersion::TlsV13, TlsVersion::TlsV13);
    ///
    /// // TLS 1.2 and 1.3 (recommended)
    /// let config = TlsConfig::new("cert.pem", "key.pem")
    ///     .tls_versions(TlsVersion::TlsV12, TlsVersion::TlsV13);
    /// ```
    ///
    /// # Compatibility Notes
    ///
    /// Limiting to TLS 1.3 only may cause compatibility issues with older clients.
    /// TLS 1.2 provides broad compatibility while maintaining strong security.
    pub fn tls_versions(mut self, min: TlsVersion, max: TlsVersion) -> Self {
        self.min_tls_version = min;
        self.max_tls_version = max;
        self
    }

    /// Builds a TLS acceptor from this configuration.
    ///
    /// This method loads the certificate and private key files, configures the TLS settings,
    /// and creates a `TlsAcceptor` that can be used to handle incoming TLS connections.
    ///
    /// # Returns
    ///
    /// Returns a `TlsAcceptor` configured with this configuration, or a `TlsError`
    /// if the configuration is invalid or files cannot be loaded.
    ///
    /// # Errors
    ///
    /// This method can fail for several reasons:
    /// - Certificate or key files not found
    /// - Invalid PEM format in certificate or key files
    /// - Certificate and private key mismatch
    /// - Insufficient file permissions
    /// - Invalid TLS configuration parameters
    ///
    /// # Examples
    ///
    /// ```
    /// use ignitia::TlsConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TlsConfig::new("server.crt", "server.key");
    /// let acceptor = config.build()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance Notes
    ///
    /// This method performs file I/O and certificate parsing, so it should typically
    /// be called once during server initialization rather than for each connection.
    pub fn build(&self) -> Result<TlsAcceptor, TlsError> {
        let certs = load_certs(&self.cert_file)?;
        let key = load_private_key(&self.key_file)?;

        let mut config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| TlsError::Config(format!("Failed to configure TLS: {}", e)))?;

        // Set ALPN protocols
        if !self.alpn_protocols.is_empty() {
            config.alpn_protocols = self.alpn_protocols.clone();
        }

        info!("TLS configuration loaded successfully");
        info!(
            "ALPN protocols: {:?}",
            self.alpn_protocols
                .iter()
                .map(|p| String::from_utf8_lossy(p))
                .collect::<Vec<_>>()
        );

        Ok(TlsAcceptor::from(Arc::new(config)))
    }

    /// Generates a self-signed certificate for development use.
    ///
    /// This method creates a self-signed X.509 certificate and corresponding private key
    /// for the specified domain. The generated files are saved as "self_signed_cert.pem"
    /// and "self_signed_key.pem" in the current directory.
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name to include in the certificate (e.g., "localhost", "example.com")
    ///
    /// # Returns
    ///
    /// Returns a tuple containing the certificate and private key as PEM-encoded strings.
    ///
    /// # Errors
    ///
    /// - `TlsError::CertGeneration` if certificate generation fails
    /// - `TlsError::Io` if file writing fails
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "self-signed")]
    /// use ignitia::TlsConfig;
    ///
    /// # #[cfg(feature = "self-signed")]
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let (cert_pem, key_pem) = TlsConfig::generate_self_signed("localhost")?;
    /// println!("Generated certificate for localhost");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Security Warning
    ///
    /// **Self-signed certificates should NEVER be used in production!**
    ///
    /// Self-signed certificates provide no identity verification and are vulnerable
    /// to man-in-the-middle attacks. They should only be used for:
    /// - Local development
    /// - Testing environments
    /// - Internal systems where trust is established through other means
    ///
    /// For production use, obtain certificates from a trusted Certificate Authority (CA).
    ///
    /// # File Output
    ///
    /// This method creates two files in the current directory:
    /// - `self_signed_cert.pem` - The certificate in PEM format
    /// - `self_signed_key.pem` - The private key in PEM format
    ///
    /// Ensure the private key file has appropriate permissions (e.g., 600) after generation.
    #[cfg(feature = "self-signed")]
    #[cfg_attr(docsrs, doc(cfg(feature = "self-signed")))]
    pub fn generate_self_signed(domain: &str) -> Result<(String, String), TlsError> {
        use rcgen::{Certificate, CertificateParams, DistinguishedName};

        let mut params = CertificateParams::new(vec![domain.to_string()]);
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(rcgen::DnType::CommonName, domain);

        let cert = Certificate::from_params(params).map_err(|e| {
            TlsError::CertGeneration(format!("Failed to generate certificate: {}", e))
        })?;

        let cert_pem = cert.serialize_pem().map_err(|e| {
            TlsError::CertGeneration(format!("Failed to serialize certificate: {}", e))
        })?;

        let key_pem = cert.serialize_private_key_pem();

        std::fs::write("self_signed_cert.pem", &cert_pem).map_err(TlsError::Io)?;
        std::fs::write("self_signed_key.pem", &key_pem).map_err(TlsError::Io)?;

        warn!(
            "Generated self-signed certificate for '{}' - DO NOT USE IN PRODUCTION",
            domain
        );

        Ok((cert_pem, key_pem))
    }
}

/// Errors that can occur during TLS configuration and operation.
///
/// This enum covers all possible error conditions when working with TLS,
/// from file I/O errors to certificate validation failures.
///
/// # Error Categories
///
/// - **I/O Errors**: File reading, writing, or permission issues
/// - **Configuration Errors**: Invalid TLS settings or rustls configuration
/// - **Certificate Errors**: Invalid, expired, or malformed certificates
/// - **Key Errors**: Invalid or mismatched private keys
/// - **Generation Errors**: Self-signed certificate creation failures
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// I/O operation failed.
    ///
    /// This error occurs when file operations fail, such as:
    /// - Certificate or key files not found
    /// - Insufficient permissions to read files
    /// - Disk space issues when writing files
    /// - Network I/O errors during TLS operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// TLS configuration is invalid.
    ///
    /// This error indicates a problem with the TLS configuration itself:
    /// - Incompatible cipher suites
    /// - Invalid protocol version combinations
    /// - Mismatched certificate and private key
    /// - Unsupported TLS features
    #[error("Configuration error: {0}")]
    Config(String),

    /// Certificate parsing or validation failed.
    ///
    /// Common causes include:
    /// - Invalid PEM format
    /// - Corrupted certificate data
    /// - Expired certificates
    /// - Certificates with invalid extensions
    /// - Missing intermediate certificates
    #[error("Certificate parsing error: {0}")]
    CertParsing(String),

    /// Private key parsing or validation failed.
    ///
    /// This can happen when:
    /// - Private key format is not supported
    /// - Private key is corrupted or invalid
    /// - Private key doesn't match the certificate
    /// - Private key is encrypted (not currently supported)
    #[error("Key parsing error: {0}")]
    KeyParsing(String),

    /// Self-signed certificate generation failed.
    ///
    /// This error occurs during development certificate generation:
    /// - Invalid domain name format
    /// - Cryptographic operation failures
    /// - Insufficient system entropy
    /// - File system write errors
    #[cfg(feature = "self-signed")]
    #[error("Certificate generation error: {0}")]
    CertGeneration(String),
}

/// Loads X.509 certificates from a PEM-encoded file.
///
/// This function reads and parses a certificate file that may contain one or more
/// certificates in PEM format. The file should contain the server certificate
/// and optionally intermediate certificates in the certificate chain.
///
/// # Arguments
///
/// * `path` - Path to the PEM-encoded certificate file
///
/// # Returns
///
/// Returns a vector of `Certificate` objects, or a `TlsError` if loading fails.
///
/// # Errors
///
/// - `TlsError::Io` if the file cannot be read
/// - `TlsError::CertParsing` if the file contains invalid PEM data
///
/// # File Format
///
/// The certificate file should contain certificates in PEM format:
///
/// ```
/// -----BEGIN CERTIFICATE-----
/// MIIBkTCB+wIJANm... (base64 encoded certificate data)
/// -----END CERTIFICATE-----
/// -----BEGIN CERTIFICATE-----
/// MIIBkTCB+wIJANm... (optional intermediate certificate)
/// -----END CERTIFICATE-----
/// ```
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "tls")]
/// # use ignitia::server::tls::load_certs;
///
/// # #[cfg(feature = "tls")]
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let certs = load_certs("server.crt")?;
/// println!("Loaded {} certificate(s)", certs.len());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
fn load_certs<P: AsRef<Path>>(path: P) -> Result<Vec<Certificate>, TlsError> {
    let file = File::open(&path).map_err(|_| {
        TlsError::Io(io::Error::new(
            ErrorKind::NotFound,
            format!("Certificate file not found: {}", path.as_ref().display()),
        ))
    })?;

    let mut reader = BufReader::new(file);
    let certs = certs(&mut reader)
        .map_err(|e| TlsError::CertParsing(format!("Failed to parse certificates: {}", e)))?;

    if certs.is_empty() {
        return Err(TlsError::CertParsing(
            "No certificates found in file".into(),
        ));
    }

    Ok(certs.into_iter().map(Certificate).collect())
}

/// Loads a private key from a PEM-encoded file.
///
/// This function reads and parses a private key file in PEM format. The private key
/// must correspond to the public key in the server certificate and be in a supported format.
///
/// # Arguments
///
/// * `path` - Path to the PEM-encoded private key file
///
/// # Returns
///
/// Returns a `PrivateKey` object, or a `TlsError` if loading fails.
///
/// # Errors
///
/// - `TlsError::Io` if the file cannot be read
/// - `TlsError::KeyParsing` if the file contains invalid or unsupported key data
///
/// # Supported Key Formats
///
/// - PKCS#8 (recommended): Modern format that supports various key types
/// - Traditional RSA private keys: Legacy format for RSA keys only
/// - EC private keys: Elliptic curve keys in traditional format
///
/// # File Format
///
/// The private key file should contain a single private key in PEM format:
///
/// ```
/// -----BEGIN PRIVATE KEY-----
/// MIIEvQIBADANBgkq... (base64 encoded key data)
/// -----END PRIVATE KEY-----
/// ```
///
/// # Security Notes
///
/// - Private key files should have restrictive permissions (e.g., 600)
/// - Encrypted private keys are not currently supported
/// - Keys should be generated with sufficient entropy and key length
/// - Consider using hardware security modules for production deployments
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "tls")]
/// # use ignitia::server::tls::load_private_key;
///
/// # #[cfg(feature = "tls")]
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let key = load_private_key("server.key")?;
/// println!("Private key loaded successfully");
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "tls")]
#[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
fn load_private_key<P: AsRef<Path>>(path: P) -> Result<PrivateKey, TlsError> {
    let file = File::open(&path).map_err(|_| {
        TlsError::Io(io::Error::new(
            ErrorKind::NotFound,
            format!("Private key file not found: {}", path.as_ref().display()),
        ))
    })?;

    let mut reader = BufReader::new(file);
    let mut keys = pkcs8_private_keys(&mut reader)
        .map_err(|e| TlsError::KeyParsing(format!("Failed to parse private key: {}", e)))?;

    if keys.is_empty() {
        return Err(TlsError::KeyParsing("No private keys found in file".into()));
    }

    if keys.len() > 1 {
        warn!("Multiple private keys found, using the first one");
    }

    Ok(PrivateKey(keys.remove(0)))
}

// Stub implementations for when TLS feature is disabled
// These allow the code to compile without the TLS feature, providing clear error messages

/// Stub TLS configuration when TLS feature is disabled.
///
/// This type is only available when the `tls` feature is disabled.
/// All methods will panic with a clear error message indicating that
/// TLS support is not compiled in.
#[cfg(not(feature = "tls"))]
#[derive(Debug, Clone)]
pub struct TlsConfig;

/// Stub TLS version enum when TLS feature is disabled.
///
/// This enum is only available when the `tls` feature is disabled.
/// It exists for API compatibility but cannot be used.
#[cfg(not(feature = "tls"))]
#[derive(Debug, Clone)]
pub enum TlsVersion {
    /// TLS 1.2 (unavailable without TLS feature)
    TlsV12,
    /// TLS 1.3 (unavailable without TLS feature)
    TlsV13,
}

/// Error type when TLS feature is disabled.
///
/// This error type is returned by TLS-related operations when the
/// `tls` feature is not enabled, providing a clear indication of
/// the missing feature.
#[cfg(not(feature = "tls"))]
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    /// TLS feature is not enabled in this build.
    ///
    /// To use TLS functionality, rebuild with the `tls` feature:
    /// ```
    /// [dependencies]
    /// ignitia = { version = "0.1", features = ["tls"] }
    /// ```
    #[error("TLS feature not enabled")]
    NotEnabled,
}

#[cfg(not(feature = "tls"))]
impl TlsConfig {
    /// Creates a stub TLS configuration.
    ///
    /// # Panics
    ///
    /// This method panics because TLS support is not compiled in.
    /// Enable the `tls` feature to use TLS functionality.
    pub fn new(_cert_file: impl Into<String>, _key_file: impl Into<String>) -> Self {
        panic!("TLS feature not enabled. Add 'tls' feature to your Cargo.toml");
    }
}
