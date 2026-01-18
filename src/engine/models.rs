use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ManifestFile {
    pub name: String,
    pub size_bytes: u64,
    pub checksum: String,
    pub download_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Manifest {
    pub version: String,
    pub files: Vec<ManifestFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalState {
    pub version: String,
}
