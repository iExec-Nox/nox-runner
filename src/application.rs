use futures_util::StreamExt;
use tracing::{error, info};

use crate::config::Config;
use crate::crypto::CryptoService;
use crate::handle_gateway::GatewayClient;
use crate::queue::QueueService;

pub struct Application {
    config: Config,
    queue_svc: QueueService,
}

impl Application {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let handle_gateway = GatewayClient::new(&config.handle_gateway_url).await?;
        let crypto_svc = CryptoService::new(&config.kms_url).await?;
        let queue_svc = QueueService::new(crypto_svc, handle_gateway);
        Ok(Application { config, queue_svc })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = async_nats::connect(&self.config.nats_url).await?;
        let mut subscriber = client.subscribe(self.config.nats_subject.clone()).await?;

        loop {
            tokio::select! {
                _ = shutdown_signal() => {
                    info!("received shutdown signal, exiting gracefully...");
                   break;
                }
                Some(message) = subscriber.next() => {
                    match self.queue_svc.handle_message(message).await {
                        Ok(_) => info!("Compute PASS"),
                        Err(e) => error!("Compute FAIL {}", e),
                    };
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
