//! NATS client module for the runner — mTLS + multi-URL + connection-state observer.

mod client;

pub use client::{ConnectionState, NatsClient};
