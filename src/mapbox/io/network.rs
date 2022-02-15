extern crate reqwest;

use super::super::super::config;
use super::super::common::async_executor::AsyncExecutor;
use super::super::common::map_error::{MapError, MapErrorTag};
use super::super::common::task::{Task, TaskReturn};
use super::super::common::task_responder::TaskResponder;
use super::super::common::types::{Threadable, ThreadableNew};
use std::io::Read;

pub struct Network {
    async_executor: AsyncExecutor,
}

// TODO: nice-to-have features: retry
impl Network {
    pub fn new(network_worker_count: usize) -> Network {
        Network {
            async_executor: AsyncExecutor::new(network_worker_count),
        }
    }

    pub fn get(&self, url: &str, responder: Threadable<dyn TaskResponder>) {
        let thread_url = url.to_string();

        let worker = ThreadableNew(move || -> TaskReturn {
            let client = reqwest::Client::new();
            let result = client
                .get(&*thread_url)
                .header(config::UBER_AUTH_HEADER, config::UBER_AUTH_TOKEN)
                .send();

            match result {
                // TODO: pass requested url back to TaskResponder
                Ok(mut res) => {
                    let mut data = vec![];
                    res.read_to_end(&mut data)
                        .expect("Network data reading error");

                    Ok(Some(data))
                }
                Err(err) => Err(MapError::new(MapErrorTag::Network, err.to_string())),
            }
        });

        self.async_executor
            .queue_task(Task::new(url.to_owned(), responder, worker));
    }
}
