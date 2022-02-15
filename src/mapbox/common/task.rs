use super::map_error::MapError;
use super::task_responder::TaskResponder;

use super::types::Threadable;

pub type TaskReturn = Result<Option<Vec<u8>>, MapError>;

pub struct Task {
    pub request_url: String, // TODO: NetworkTask
    pub responder: Threadable<dyn TaskResponder>,
    pub worker: Threadable<dyn Fn() -> TaskReturn + Send + Sync>,
}

impl Task {
    pub fn new(
        request_url: String,
        responder: Threadable<dyn TaskResponder>,
        worker: Threadable<dyn Fn() -> TaskReturn + Send + Sync>,
    ) -> Task {
        Task {
            request_url,
            responder,
            worker,
        }
    }
}
