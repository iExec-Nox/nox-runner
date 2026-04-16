use alloy_primitives::hex;
use alloy_signer_local::PrivateKeySigner;
use axum::{Router, routing::get};
use axum_prometheus::{Handle, MakeDefaultHandle, PrometheusMetricLayerBuilder, metrics::counter};
use futures_util::StreamExt;
use tracing::{error, info};

use crate::config::Config;
use crate::events::TransactionMessage;
use crate::handlers;
use crate::handles::{crypto::CryptoService, gateway::GatewayClient};
use crate::queue::QueueService;
use crate::rpc::NoxClient;

pub struct Application {
    config: Config,
    queue_svc: QueueService,
}

impl Application {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let nox_rpc = NoxClient::new(&config.rpc_url, config.nox_compute_contract_address).await?;
        let protocol_key_bytes = nox_rpc.get_kms_public_key().await?;
        let handle_gateway_signer_address = nox_rpc.get_gateway_address().await?;
        info!("Handle Gateway signer address: {handle_gateway_signer_address}");

        let crypto_svc = CryptoService::new(protocol_key_bytes).await?;
        let mut wallet_key_bytes = [0u8; 32];
        wallet_key_bytes.copy_from_slice(&hex::decode(&config.wallet_key)?);
        let signer = PrivateKeySigner::from_bytes(&wallet_key_bytes.into())?;
        let handle_gateway = GatewayClient::new(
            config.chain_id,
            &config.handle_gateway_url,
            handle_gateway_signer_address,
            signer,
        )
        .await?;
        let queue_svc = QueueService::new(crypto_svc, handle_gateway);
        Ok(Application { config, queue_svc })
    }

    /// Connects to existing NATS stream to consume messages.
    ///
    /// Received messages are deserialized as messages representing transactions.
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = async_nats::connect(&self.config.nats.url).await?;
        let jetstream = async_nats::jetstream::new(client);
        let stream = jetstream.get_stream(&self.config.nats.stream_name).await?;
        let consumer = stream
            .get_or_create_consumer(
                &self.config.nats.consumer_name,
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.config.nats.consumer_name.clone()),
                    max_deliver: self.config.nats.consumer_max_deliver,
                    max_ack_pending: self.config.nats.max_ack_pending,
                    max_batch: self.config.nats.max_batch,
                    ..Default::default()
                },
            )
            .await?;
        let mut subscriber = consumer.messages().await?;

        let prometheus_layer = PrometheusMetricLayerBuilder::new()
            .with_allow_patterns(&["/", "/health", "/metrics"])
            .build();
        let metrics_handle = Handle::make_default_handle(Handle::default());

        let app = Router::new()
            .route("/", get(handlers::root))
            .route("/health", get(handlers::health_check))
            .route("/metrics", get(handlers::metrics))
            .fallback(handlers::not_found)
            .layer(prometheus_layer)
            .with_state(metrics_handle);
        let binding_address = self.config.binding_address();
        info!("starting TCP server listening on {binding_address}");
        let listener = tokio::net::TcpListener::bind(binding_address).await?;
        tokio::spawn(async move { axum::serve(listener, app).await });

        info!("entering main loop to receive messages from NATS JetStream");
        loop {
            tokio::select! {
                _ = shutdown_signal() => {
                    info!("received shutdown signal, exiting gracefully...");
                   break;
                }
                Some(message) = subscriber.next() => {
                    match message {
                        Ok(msg) => {
                            let transaction_message = match serde_json::from_slice::<TransactionMessage>(&msg.payload) {
                                Ok(v) => v,
                                Err(e) => {
                                    error!("Failed to deserialize message: {e}");
                                    match msg.ack().await {
                                        Ok(_) => info!("ACK sent for invalid message"),
                                        Err(ack_err) => error!("ACK could not be sent for invalid message: {ack_err}"),
                                    }
                                    continue;
                                }
                            };
                            counter!("nox_runner.transaction.received").increment(1);
                            counter!("nox_runner.transaction.block_number").absolute(transaction_message.block_number);
                            let transaction_hash = transaction_message.transaction_hash.clone();
                            match self.queue_svc.handle_message(transaction_message).await {
                                Ok(_) => {
                                    info!(transaction_hash, "Compute PASS");
                                    match msg.ack().await {
                                        Ok(_) => {
                                            counter!("nox_runner.transaction.result", "STATUS" => "SUCCESS").increment(1);
                                            info!(transaction_hash, "ACK sent")
                                        }
                                        Err(ack_err) => {
                                            counter!("nox_runner.transaction.result", "STATUS" => "NOT_ACK").increment(1);
                                            error!(transaction_hash, "ACK could not be sent {ack_err}")
                                        }
                                    };
                                },
                                Err(e) => {
                                    counter!("nox_runner.transaction.result", "STATUS" => "FAILURE").increment(1);
                                    error!(transaction_hash, "Compute FAIL {e}")
                                }
                            }
                            self.queue_svc.reset_cache();
                        },
                        Err(e) => error!("Failed to pull message {e}"),
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT");
        }
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
}
