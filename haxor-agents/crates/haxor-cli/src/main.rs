use clap::{Parser, Subcommand};
use haxor_core::crds::{Database, DatabaseRole, DbUser};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
    api::{Patch, PatchParams},
    Api, Client, CustomResourceExt as _,
};

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup { crds_only: Option<bool> },
}

fn main() {
    let app = App::parse();

    match app.command {
        Commands::Setup { crds_only } => setup(crds_only.unwrap_or(false)),
    }
}

pub fn setup(crds_only: bool) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Could not spawn tokio runtime");

    rt.block_on(setup_async(crds_only));
}

async fn setup_async(crds_only: bool) {
    let client = Client::try_default()
        .await
        .expect("Could not create client");

    apply_crds(Api::all(client)).await;
}

async fn apply_crds(crds: Api<CustomResourceDefinition>) {
    let pp = PatchParams::apply("haxor");

    let db_apply = Patch::Apply(Database::crd());
    let db_patch = crds.patch(Database::crd_name(), &pp, &db_apply);

    let db_role_apply = Patch::Apply(DatabaseRole::crd());
    let db_role_patch = crds.patch(DatabaseRole::crd_name(), &pp, &db_role_apply);

    let cluster_db_role_apply = Patch::Apply(DatabaseRole::crd());
    let cluster_db_role_patch = crds.patch(DatabaseRole::crd_name(), &pp, &cluster_db_role_apply);

    let (db_result, db_role_result, cluster_db_role_result) =
        tokio::join!(db_patch, db_role_patch, cluster_db_role_patch);

    if let Err(ex) = db_result {
        eprintln!("Could not install Database CRD: {ex}");
    }

    if let Err(ex) = db_role_result {
        eprintln!("Could not install DatabaseRole CRD: {ex}");
    }

    if let Err(ex) = cluster_db_role_result {
        eprintln!("Could not install ClusterDatabaseRole CRD: {ex}");
    }
}
