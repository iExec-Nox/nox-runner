use tracing::info;

pub struct Application {}

impl Application {
    pub fn new() -> Self {
        Application {}
    }

    pub async fn run(&self) {
        info!("Hello, world!");
    }
}
