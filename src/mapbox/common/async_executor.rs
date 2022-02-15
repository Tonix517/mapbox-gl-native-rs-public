use super::task::Task;
use threadpool::ThreadPool;

pub struct AsyncExecutor {
    pool: ThreadPool,
}

impl AsyncExecutor {
    pub fn new(thread_count: usize) -> AsyncExecutor {
        AsyncExecutor {
            pool: ThreadPool::new(thread_count),
        }
    }

    pub fn queue_task(&self, task: Task) {
        self.pool.execute(move || {
            let ret = task.worker.lock().unwrap()();
            let mut responder = task.responder.lock().unwrap();
            match ret {
                Ok(data) => responder.on_task_success(task.request_url, data),
                Err(err) => responder.on_task_failure(err),
            }
        });
    }
}
