use super::map_error::MapError;

pub trait TaskResponder: Send {
    // TODO: NetworkTaskResponder
    fn on_task_success(&mut self, url: String, data: Option<Vec<u8>>);
    fn on_task_failure(&self, map_error: MapError);
}
