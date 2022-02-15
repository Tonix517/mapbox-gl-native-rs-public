pub trait ManifestObserver: Send {
    fn on_manifest_loaded(&mut self, name: String, url_template: String, avail_zooms: Vec<u64>);
    fn on_manifest_failed(&self, name: String);
}
