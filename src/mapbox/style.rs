use super::common::map_error::MapError;
use super::common::task_responder::TaskResponder;
use super::io::resource::Resource;
use super::style_model::StyleModel;

use super::manifest::Manifest;

use crate::mapbox::manifest_observer::ManifestObserver;
use serde_json::{Map, Value};
use std::cell::RefCell;

use super::common::types::{Threadable, ThreadableNew};

struct StyleImpl {
    style_model: StyleModel,
    manifests: RefCell<Vec<Manifest>>,
    obs: Option<Threadable<dyn ManifestObserver>>,
}

impl StyleImpl {
    fn new() -> StyleImpl {
        StyleImpl {
            style_model: StyleModel::new(),
            manifests: RefCell::new(vec![]),
            obs: None,
        }
    }

    fn load_manifest_items(&self, sources: &Map<String, Value>) {
        for i in sources.iter() {
            println!("> {:?}", i);

            let manifest_instance = Manifest::new(i.0.to_string());
            manifest_instance.load_manifest(i.0.to_string(), i.1);

            if self.obs.is_some() {
                let obs = self.obs.as_ref().unwrap().clone();
                manifest_instance.add_manifest_observer(obs);
            }

            self.manifests.borrow_mut().push(manifest_instance);
        }
    }

    pub fn add_manifest_observer(&mut self, obs: Threadable<dyn ManifestObserver>) {
        self.obs = Some(obs);

        if !self.manifests.borrow_mut().is_empty() {
            for manifest in self.manifests.borrow_mut().iter_mut() {
                let obs = self.obs.as_ref().unwrap().clone();
                manifest.add_manifest_observer(obs);
            }
        }
    }
}

impl TaskResponder for StyleImpl {
    fn on_task_success(&mut self, _url: String, data: Option<Vec<u8>>) {
        println!("Yikes: Style Load Succeeded");

        match data {
            Some(str) => {
                // Parse stylesheet
                let str_ret = String::from_utf8(str).unwrap();
                let style: Value = serde_json::from_str(&str_ret).unwrap();
                self.style_model = StyleModel::parse(style);

                // Load Sources
                self.load_manifest_items(&self.style_model.sources);
            }
            None => {
                println!("Error: empty stylesheet loaded");
            }
        }
    }

    fn on_task_failure(&self, map_error: MapError) {
        println!("Error: Style Load Failed {}", map_error);
    }
}

pub struct Style {
    style_impl: Threadable<StyleImpl>,
    resource: Resource,
    // TODO: there're quite a bit of get functions to expose
    //       like style configs, manifest info etc.
}

impl Style {
    pub fn new() -> Style {
        let style_impl = ThreadableNew(StyleImpl::new());
        let resource = Resource::new(1);
        Style {
            style_impl,
            resource,
        }
    }

    pub fn load_style_with_url(&self, url: &'static str) {
        let responder = self.style_impl.clone();
        self.resource.get(url, responder);
    }

    pub fn add_manifest_observer(&mut self, obs: Threadable<dyn ManifestObserver>) {
        self.style_impl.lock().unwrap().add_manifest_observer(obs);
    }
}
