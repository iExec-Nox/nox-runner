//! NATS client with JetStream support

use async_nats::jetstream::{self, Context as JetStreamContext};
use async_nats::{ConnectOptions, Event};
use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info, warn};

use crate::config::NatsConfig;

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

        for (label, path) in [
            ("ca", &config.tls.ca_path),
            ("cert", &config.tls.cert_path),
            ("key", &config.tls.key_path),
        ] {
            if !path.is_file() {
                return Err(
                    format!("{label} path is not a regular file: {}", path.display()).into(),
                );
            }
        }

        let options = ConnectOptions::new()
            .event_callback(move |event| {
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
            })
            .add_root_certificates(config.tls.ca_path.clone())
            .add_client_certificate(config.tls.cert_path.clone(), config.tls.key_path.clone())
            .require_tls(true);

        info!(
            urls = ?config.urls,
            "Connecting to NATS cluster via mTLS"
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
