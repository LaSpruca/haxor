use anyhow::anyhow;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use k8s_openapi::{
    api::{
        apps::v1::StatefulSet,
        core::v1::{Secret, Service},
    },
    ByteString,
};
use kube::{core::ErrorResponse, Api, Client};
use rand::{distr::Alphanumeric, Rng};
use tracing::{debug, error, info, span, Level};

use crate::templates;

pub struct DeploymentManager {
    client: Client,
    password: String,
}

impl std::fmt::Debug for DeploymentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeploymentManager")
            .field("password", &self.password)
            .finish()
    }
}

impl DeploymentManager {
    pub async fn new(client: &Client) -> anyhow::Result<Self> {
        let password = match Self::get_or_generate_password(client).await {
            Ok(val) => val,
            Err(ex) => {
                error!("Could not get or create postgres password {ex:?}");
                return Err(ex);
            }
        };

        return Ok(Self {
            client: client.clone(),
            password,
        });
    }

    async fn get_or_generate_password(client: &Client) -> anyhow::Result<String> {
        let secrets: Api<Secret> = Api::namespaced(client.clone(), "haxor");
        match secrets.get("postgres-password").await {
            // If we can get the password
            Ok(secret)
                if secret
                    .data
                    .as_ref()
                    .is_some_and(|data| data.get("postgres-password").is_some()) =>
            {
                let data = secret.data.unwrap();
                let ByteString(secret_data) = data.get("postgres-password").unwrap();
                let secret_value = BASE64_STANDARD.decode(secret_data)?;
                let password = String::from_utf8(secret_value)?;

                return Ok(password);
            }

            Ok(_) | Err(kube::Error::Api(ErrorResponse { code: 404, .. })) => {}
            Err(ex) => {
                return Err(anyhow!(ex));
            }
        };

        // Create a new password and save it to kubernetes as a secret
        let password = rand::rng()
            .sample_iter(Alphanumeric)
            .map(char::from)
            .take(50)
            .collect::<String>();

        let encoded_password = BASE64_STANDARD.encode(&password);

        // Therotically templating here should not fail
        let password_secret_str = templates::root_password_secret(encoded_password.as_str())
            .expect("FAITAL: Could not render the postgres password");

        let password_secret: Secret =
            serde_yaml::from_str(&password_secret_str).expect("Could not parse password secret");

        match secrets
            .create(&kube::api::PostParams::default(), &password_secret)
            .await
        {
            Ok(_) => {}
            Err(ex) => {
                error!("Could not upload password {ex}");
                return Err(anyhow!(ex));
            }
        };

        return Ok(password);
    }

    pub async fn deploy(&self) -> anyhow::Result<()> {
        span!(
            Level::TRACE,
            "haxor-postgres-agent::DeploymentManager::deploy"
        );

        let stateful_sets: Api<StatefulSet> = Api::namespaced(self.client.clone(), "haxor");
        let services: Api<Service> = Api::namespaced(self.client.clone(), "haxor");

        // Check to see if we are already deployed
        match stateful_sets.get("postgres").await {
            Ok(_) => {
                info!("StatefulSet postgres found in namespace haxor, skipping deployment");
                return Ok(());
            }
            Err(kube::Error::Api(ex)) if ex.code == 404 => {}
            ex @ _ => {
                error!("Error getting stateful sets {ex:?}");
                return Err(anyhow!("Could not create database deployment"));
            }
        };

        let service_str = templates::service().expect("Could not render service template");
        let stateful_set_str =
            templates::stateful_set().expect("Could not render service template");

        // We expect here because this should never fail in the actual environment
        let service = serde_yaml::from_str(&service_str).expect("Invalid service.yaml");
        let stateful_set =
            serde_yaml::from_str(&stateful_set_str).expect("Invalid statufulservice.yaml");

        match services.create(&Default::default(), &service).await {
            Ok(_) => {}
            Err(ex) => {
                error!("Could not create postgres service {ex:?}");
                return Err(anyhow!(ex));
            }
        };

        match stateful_sets
            .create(&Default::default(), &stateful_set)
            .await
        {
            Ok(val) => {
                debug!("Created {:#?}", val);
            }
            Err(ex) => {
                error!("Could not create postgres stateful set {ex:?}");
                match services
                    .delete(&service.metadata.name.unwrap(), &Default::default())
                    .await
                {
                    Ok(_) => {}
                    Err(ex) => {
                        error!("Could not remove service {ex:?}");
                    }
                };
                return Err(anyhow!(ex));
            }
        };

        return Ok(());
    }
}
