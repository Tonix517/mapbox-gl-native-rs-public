use std::sync::{Arc, Mutex};
use std::ops::Deref;

// TODO: still a good idea to create struct Threadable implementing Deref trait
//       so that Threadable users don't have to lock().unwrap()

pub type Threadable<T> = Arc<Mutex<T>>;

pub fn ThreadableNew<T>(t: T) -> Threadable<T> {
    Arc::new(Mutex::new(t))
}

//

pub struct ThreadItem<T> {
    value: Arc<Mutex<T>>,
}

impl<T> ThreadItem<T> {
    pub fn new(t: T) -> Self {
        ThreadItem {
            value: Arc::new(Mutex::new(t))
        }
    }
}

//impl<T> Deref for ThreadItem<T> {
//}