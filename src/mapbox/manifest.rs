use super::manifest_model::{ManifestModel, ManifestType};
use super::manifest_observer::ManifestObserver;

use super::io::resource::Resource;

use super::common::map_error::MapError;
use super::common::task_responder::TaskResponder;
use super::common::types::{Threadable, ThreadableNew};

use serde_json::{Map, Value};

struct ManifestImpl {
    pub name: String,
    pub data: ManifestModel,
    obs: Option<Threadable<dyn ManifestObserver>>,
}

impl ManifestImpl {
    fn new(name: String) -> ManifestImpl {
        ManifestImpl {
            name,
            data: ManifestModel::new(),
            obs: None,
        }
    }

    pub fn add_manifest_observer(&mut self, obs: Threadable<dyn ManifestObserver>) {
        self.obs = Some(obs);
    }
}

impl TaskResponder for ManifestImpl {
    fn on_task_success(&mut self, _url: String, data: Option<Vec<u8>>) {
        match data {
            Some(str) => {
                let str_ret = String::from_utf8(str).unwrap();
                println!("== Manifest {}: {:?}", &self.name, str_ret);
                let manifest: Value = serde_json::from_str(&str_ret).unwrap();

                self.data = ManifestModel::parse(&self.data.r#type, manifest);
                println!("== Parsed Manifest {}: {:?}", &self.name, &self.data);

                if self.obs.is_some() {
                    let obs = self.obs.as_ref().unwrap();
                    let url_template = self.data.tiles[0].clone();
                    let avail_zooms = self.data.tilezooms.clone();
                    obs.lock().unwrap().on_manifest_loaded(
                        self.name.clone(),
                        url_template,
                        avail_zooms,
                    );
                }
            }
            None => {
                println!("Error: empty manifest loaded");
            }
        }
    }

    fn on_task_failure(&self, map_error: MapError) {
        println!("Error: Manifest Load Failed {}", map_error);

        if self.obs.is_some() {
            let obs = self.obs.as_ref().unwrap();
            obs.lock().unwrap().on_manifest_failed(self.name.clone());
        }
    }
}

//

pub struct Manifest {
    manifest_impl: Threadable<ManifestImpl>,
    resource: Resource,
}

impl Manifest {
    pub fn new(name: String) -> Manifest {
        let manifest_impl = ThreadableNew(ManifestImpl::new(name));
        let resource = Resource::new(4);
        Manifest {
            manifest_impl,
            resource,
        }
    }

    pub fn load_manifest(&self, name: String, value: &Value) {
        let data: Map<String, Value> = value.as_object().unwrap().to_owned();
        let r#type = data.get("type").unwrap().as_str().unwrap();
        println!("== Loading manifest {} of type {}", name, r#type);

        if data.contains_key("url") {
            let url = data.get("url").unwrap().as_str().unwrap();
            let url = self.convert_uber_url(url);
            println!("- URL {}", url);

            let responder = self.manifest_impl.clone();
            self.resource.get(&url, responder);
        } else {
            let mut m_type = ManifestType::Vector;
            match r#type {
                "raster" => m_type = ManifestType::Raster,
                _ => {}
            }
            self.manifest_impl.lock().unwrap().data =
                ManifestModel::parse(&m_type, serde_json::value::Value::Object(data));
        }
    }

    // The URL hack
    fn convert_uber_url(&self, uri: &str) -> String {
        const TMP_URI_PREFIX: &str = "/tile-discovery-api";
        const UB_DOMAIN: &str = "REMOVED";
        const UB_ENDPOINT: &str = "/rt/msd";
        let mut url: String = uri.to_owned();

        if url.starts_with(TMP_URI_PREFIX) {
            url = format!(
                "{}{}{}",
                UB_DOMAIN,
                UB_ENDPOINT,
                url[TMP_URI_PREFIX.len()..].to_owned()
            );
        }

        url
    }

    pub fn add_manifest_observer(&self, obs: Threadable<dyn ManifestObserver>) {
        self.manifest_impl
            .lock()
            .unwrap()
            .add_manifest_observer(obs);
    }
}
