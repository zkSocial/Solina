use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub struct SolinaConfig {
    mempool_capacity: usize,
    storage_file_path: PathBuf,
    socket_address: SocketAddr,
    auth_credential_timeout: u64,
}

impl SolinaConfig {
    pub fn new<P: AsRef<Path>>(
        mempool_capacity: usize,
        storage_file_path: P,
        socket_address: SocketAddr,
        auth_credential_timeout: u64,
    ) -> Self {
        Self {
            mempool_capacity,
            storage_file_path: storage_file_path.as_ref().to_path_buf(),
            socket_address,
            auth_credential_timeout,
        }
    }

    pub fn mempool_capacity(&self) -> usize {
        self.mempool_capacity
    }

    pub fn storage_file_path(&self) -> &PathBuf {
        &self.storage_file_path
    }

    pub fn socket_address(&self) -> SocketAddr {
        self.socket_address
    }

    pub fn auth_credential_timeout(&self) -> u64 {
        self.auth_credential_timeout
    }
}

impl Default for SolinaConfig {
    fn default() -> Self {
        Self {
            mempool_capacity: 5,
            storage_file_path: PathBuf::from("solina-data.sqlite"),
            socket_address: "127.0.0.1:3000".parse().unwrap(),
            auth_credential_timeout: 360,
        }
    }
}
