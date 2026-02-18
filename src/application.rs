use futures_util::StreamExt;
use tracing::{error, info};

use crate::config::Config;
use crate::crypto::CryptoService;
use crate::events::TransactionMessage;
use crate::handle_gateway::GatewayClient;
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
        let handle_gateway = GatewayClient::new(&config.handle_gateway_url).await?;
        let crypto_svc = CryptoService::new(protocol_key_bytes).await?;
        let queue_svc = QueueService::new(crypto_svc, handle_gateway);
        Ok(Application { config, queue_svc })
    }

    /// Connects to existing NATS stream to consume messages.
    ///
    /// Received messages are deserialized as messages representing transactions.
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = async_nats::connect(&self.config.nats_url).await?;
        let jetstream = async_nats::jetstream::new(client);
        let stream = jetstream.get_stream(&self.config.nats_stream_name).await?;
        let consumer = stream
            .get_or_create_consumer(
                &self.config.nats_consumer_name,
                async_nats::jetstream::consumer::pull::Config {
                    durable_name: Some(self.config.nats_consumer_name.clone()),
                    max_deliver: self.config.nats_consumer_max_deliver,
                    ..Default::default()
                },
            )
            .await?;
        let mut subscriber = consumer.messages().await?;

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
                            let transaction_hash = transaction_message.transaction_hash.clone();
                            match self.queue_svc.handle_message(transaction_message).await {
                                Ok(_) => {
                                    info!(transaction_hash, "Compute PASS");
                                    match msg.ack().await {
                                        Ok(_) => info!(transaction_hash, "ACK sent"),
                                        Err(ack_err) => error!(transaction_hash, "ACK could not be sent {ack_err}"),
                                    };
                                },
                                Err(e) => error!(transaction_hash, "Compute FAIL {e}"),
                            }
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
