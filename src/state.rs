#[derive(Clone)]
pub struct AppState {
    pub root_dir: String,
    pub exclude_files: Vec<String>,
    pub url_prefix: String,
}

impl AppState {
    pub fn new(root_dir: String, exclude_files: Vec<String>, url_prefix: String) -> Self {
        Self {
            root_dir,
            exclude_files,
            url_prefix,
        }
    }
}
