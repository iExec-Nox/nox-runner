//! NATS client with JetStream support

use async_nats::jetstream::{self, Context as JetStreamContext};
use async_nats::rustls::pki_types::pem::PemObject;
use async_nats::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use async_nats::rustls::{ClientConfig, RootCertStore};
use async_nats::{ConnectOptions, Event};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info, warn};

use crate::config::{NatsConfig, TlsConfig};

/// Connection state for NATS client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Disconnected => write!(f, "Disconnected"),
        }
    }
}

/// NATS client with JetStream support
pub struct NatsClient {
    jetstream: Arc<JetStreamContext>,
    state_rx: watch::Receiver<ConnectionState>,
}

impl NatsClient {
    /// Connect to NATS server
    pub async fn connect(
        config: &NatsConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let (state_tx, state_rx) = watch::channel(ConnectionState::Disconnected);

        let state_tx_clone = state_tx.clone();

        let mut options = ConnectOptions::new().event_callback(move |event| {
            let state_tx = state_tx_clone.clone();
            async move {
                match event {
                    Event::Connected => {
                        info!("NATS connected");
                        let _ = state_tx.send(ConnectionState::Connected);
                    }
                    Event::Disconnected => {
                        warn!("NATS disconnected");
                        let _ = state_tx.send(ConnectionState::Disconnected);
                    }
                    Event::ServerError(err) => error!(error = %err, "NATS server error"),
                    Event::ClientError(err) => error!(error = %err, "NATS client error"),
                    Event::LameDuckMode => warn!("NATS server in lame duck mode"),
                    Event::SlowConsumer(sid) => {
                        warn!(subscription_id = sid, "NATS slow consumer")
                    }
                    _ => {}
                }
            }
        });

        if config.tls.enabled {
            let tls_config = build_rustls_client_config(&config.tls)?;
            options = options.require_tls(true).tls_client_config(tls_config);
        }

        info!(
            urls = ?config.urls,
            tls = config.tls.enabled,
            "Connecting to NATS cluster"
        );

        let client = options
            .connect(&config.urls[..])
            .await
            .map_err(|e| format!("Failed to connect to NATS cluster {:?}: {}", config.urls, e))?;

        let _ = state_tx.send(ConnectionState::Connected);

        let jetstream = jetstream::new(client);

        info!("NATS connected successfully");

        Ok(Self {
            jetstream: Arc::new(jetstream),
            state_rx,
        })
    }

    /// Get the JetStream context
    pub fn jetstream(&self) -> Arc<JetStreamContext> {
        Arc::clone(&self.jetstream)
    }

    /// Get a receiver for connection state changes
    pub fn state_receiver(&self) -> watch::Receiver<ConnectionState> {
        self.state_rx.clone()
    }
}

/// Normalizes a PEM string that may have been collapsed into a single line.
///
/// Handles two common inline injection patterns:
/// - Spaces replacing newlines: `"-----BEGIN CERTIFICATE----- MIIFxx... -----END CERTIFICATE-----"`
/// - Literal `\n` sequences from env vars or YAML double-quoted strings
fn normalize_pem(pem: &str) -> String {
    // Resolve literal \n sequences (env var injection)
    let pem = pem.replace("\\n", "\n");
    // If already has proper newlines after markers, return as-is
    if pem.contains("-----\n") {
        return pem;
    }
    // Restore newlines around PEM section markers
    // Base64 alphabet has no spaces, so replacing ` -----`/`----- ` is unambiguous
    let normalized = pem
        .replace("----- ", "-----\n")
        .replace(" -----", "\n-----");
    if normalized.ends_with('\n') {
        normalized
    } else {
        normalized + "\n"
    }
}

/// Build an in-memory rustls `ClientConfig` from PEM strings supplied via env vars.
fn build_rustls_client_config(
    tls: &TlsConfig,
) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
    for (label, value) in [("ca", &tls.ca), ("cert", &tls.cert), ("key", &tls.key)] {
        if value.trim().is_empty() {
            return Err(format!(
                "TLS enabled but `{label}` PEM content is empty (set NOX_RUNNER_NATS__TLS__{} env var)",
                label.to_uppercase()
            )
            .into());
        }
    }

    let ca = normalize_pem(&tls.ca);
    let cert = normalize_pem(&tls.cert);
    let key = normalize_pem(&tls.key);

    let mut roots = RootCertStore::empty();
    for cert_der in CertificateDer::pem_slice_iter(ca.as_bytes()) {
        let cert_der = cert_der.map_err(|e| format!("Failed to parse CA PEM: {e}"))?;
        roots
            .add(cert_der)
            .map_err(|e| format!("Failed to add CA cert to root store: {e}"))?;
    }
    if roots.is_empty() {
        return Err("No CA certificates found in PEM content".into());
    }

    let cert_chain: Vec<CertificateDer<'static>> = CertificateDer::pem_slice_iter(cert.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to parse client cert PEM: {e}"))?;
    if cert_chain.is_empty() {
        return Err("No client certificates found in PEM content".into());
    }

    let private_key = PrivateKeyDer::from_pem_slice(key.as_bytes())
        .map_err(|e| format!("Failed to parse client key PEM: {e}"))?;

    ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(cert_chain, private_key)
        .map_err(|e| format!("Failed to build rustls client config: {e}").into())
}
