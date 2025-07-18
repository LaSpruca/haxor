use kube::{CustomResource, CustomResourceExt, Resource};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, JsonSchema, Debug, Deserialize, Serialize, Clone)]
#[kube(
    group = "haxor.laspruca.nz",
    version = "v1",
    kind = "Database",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSpec {
    pub users: Vec<DbUser>,
}

#[derive(Debug, Deserialize, Serialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DbUser {
    pub name: String,
    pub role: String,
}

#[derive(CustomResource, JsonSchema, Debug, Deserialize, Serialize, Clone)]
#[kube(
    group = "haxor.laspruca.nz",
    version = "v1",
    kind = "DatabaseRole",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRoleSpec {
    apply: String,
}

#[derive(CustomResource, JsonSchema, Debug, Deserialize, Serialize, Clone)]
#[kube(
    group = "haxor.laspruca.nz",
    version = "v1",
    kind = "ClusterDatabaseRole"
)]
#[serde(rename_all = "camelCase")]
pub struct ClusterDatabaseRoleSpec {
    apply: String,
}
