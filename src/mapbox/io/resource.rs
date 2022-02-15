use super::super::common::task_responder::TaskResponder;
use super::network::Network;

use super::super::common::types::Threadable;

pub struct Resource {
    network: Network,
}

impl Resource {
    pub fn new(network_worker_count: usize) -> Resource {
        Resource {
            network: Network::new(network_worker_count),
        }
    }

    pub fn get(&self, uri: &str, responder: Threadable<dyn TaskResponder>) {
        // TODO: enable disk cache: https://lib.rs/crates/lru-disk-cache ?
        self.network.get(uri, responder);
    }
}
