use deployment_manager::DeploymentManager;
use kube::Client;
use std::process::ExitCode;
use tracing::{debug, error, span, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt as _, EnvFilter};

mod deployment_manager;
mod templates;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    span!(Level::TRACE, "haxor-postgres-agent::main");

    let client = match Client::try_default().await {
        Ok(val) => val,
        Err(ex) => {
            error!("Could not connect to kubernetes API: {ex}");
            return ExitCode::FAILURE;
        }
    };

    let deployment_manager = match DeploymentManager::new(&client).await {
        Ok(val) => val,
        Err(ex) => {
            error!("Error attempting to setup database manager {ex}");
            return ExitCode::FAILURE;
        }
    };

    debug!("{deployment_manager:?}");

    if let Err(ex) = deployment_manager.deploy().await {
        error!("Could not deploy postgres: {:?}", ex);
    }

    return ExitCode::SUCCESS;
}
