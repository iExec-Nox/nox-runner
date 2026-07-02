use opentelemetry::trace::TracerProvider;
use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, propagation::TraceContextPropagator,
    trace::SdkTracerProvider,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use validator::Validate;

use crate::application::Application;
use crate::config::Config;

mod application;
mod compute;
mod config;
mod events;
mod handlers;
mod handles;
mod nats;
mod observability;
mod queue;
mod rpc;

fn init_logs(
    url: &str,
    resource: Resource,
) -> Result<SdkLoggerProvider, Box<dyn std::error::Error>> {
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/logs", url))
        .with_protocol(Protocol::HttpBinary)
        .build()?;

    Ok(SdkLoggerProvider::builder()
        .with_simple_exporter(exporter)
        .with_resource(resource)
        .build())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_nats::rustls::crypto::ring::default_provider()
        .install_default()
        .unwrap_or_else(|_| error!("Failed to install rustls ring crypto provider"));

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let config = Config::load().inspect_err(|e| error!("Failed to load configuration: {e}"))?;
    config
        .validate()
        .inspect_err(|e| error!("Invalid configuration: {e}"))?;

    info!("Configuration loaded");

    let tracing_registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer());

    if config.otel.enabled {
        let exporter = SpanExporter::builder()
            .with_http()
            .with_endpoint(format!("{}/v1/traces", config.otel.url))
            .with_protocol(Protocol::HttpBinary)
            .build()?;

        let resource = Resource::builder()
            .with_attribute(KeyValue::new("service.name", "nox-runner"))
            .build();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .with_resource(resource.clone())
            .build();

        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(provider.clone());

        let telemetry_layer =
            tracing_opentelemetry::layer().with_tracer(provider.tracer("nox-runner"));

        let log_provider = init_logs(&config.otel.url, resource.clone())?;

        tracing_registry
            .with(telemetry_layer)
            .with(OpenTelemetryTracingBridge::new(&log_provider))
            .init();
    } else {
        tracing_registry.init();
    }

    Application::new(config).await?.run().await?;
    Ok(())
}
