use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc, time::SystemTime};
use tokio::sync::{broadcast, Mutex};

pub(crate) type SharedMarkdownState = Arc<Mutex<MarkdownState>>;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub(crate) enum ClientMessage {
    Ping,
    RequestRefresh,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Reload,
    Pong,
}

pub(crate) struct TrackedFile {
    pub(crate) path: PathBuf,
    pub(crate) last_modified: SystemTime,
    pub(crate) html: String,
}

pub(crate) struct MarkdownState {
    pub(crate) base_dir: PathBuf,
    pub(crate) tracked_files: HashMap<String, TrackedFile>,
    pub(crate) is_directory_mode: bool,
    pub(crate) change_tx: broadcast::Sender<ServerMessage>,
    pub(crate) mdlive_dir: Option<PathBuf>,
}

impl MarkdownState {
    pub(crate) fn new(
        base_dir: PathBuf,
        file_paths: Vec<PathBuf>,
        is_directory_mode: bool,
    ) -> Result<Self> {
        let (change_tx, _) = broadcast::channel::<ServerMessage>(16);

        let mut tracked_files = HashMap::new();
        for file_path in file_paths {
            let metadata = fs::metadata(&file_path)?;
            let last_modified = metadata.modified()?;
            let content = fs::read_to_string(&file_path)?;
            let html = Self::markdown_to_html(&content)?;

            let canonical = file_path.canonicalize().unwrap_or(file_path);
            let key = canonical
                .strip_prefix(&base_dir)
                .unwrap_or(&canonical)
                .to_string_lossy()
                .to_string();

            tracked_files.insert(
                key,
                TrackedFile {
                    path: canonical,
                    last_modified,
                    html,
                },
            );
        }

        let dir = base_dir.join(".mdlive");
        let history = dir.join("history");
        let _ = fs::create_dir_all(&history);
        let mdlive_dir = Some(dir);

        Ok(MarkdownState {
            base_dir,
            tracked_files,
            is_directory_mode,
            change_tx,
            mdlive_dir,
        })
    }

    pub(crate) fn show_navigation(&self) -> bool {
        self.is_directory_mode
    }

    pub(crate) fn get_sorted_filenames(&self) -> Vec<String> {
        let mut filenames: Vec<_> = self.tracked_files.keys().cloned().collect();
        filenames.sort();
        filenames
    }

    pub(crate) fn refresh_file(&mut self, filename: &str) -> Result<()> {
        if let Some(tracked) = self.tracked_files.get_mut(filename) {
            let metadata = fs::metadata(&tracked.path)?;
            let current_modified = metadata.modified()?;

            if current_modified > tracked.last_modified {
                let content = fs::read_to_string(&tracked.path)?;
                tracked.html = Self::markdown_to_html(&content)?;
                tracked.last_modified = current_modified;
            }
        }

        Ok(())
    }

    pub(crate) fn add_tracked_file(&mut self, file_path: PathBuf) -> Result<()> {
        let key = file_path
            .strip_prefix(&self.base_dir)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        if self.tracked_files.contains_key(&key) {
            return Ok(());
        }

        let metadata = fs::metadata(&file_path)?;
        let content = fs::read_to_string(&file_path)?;

        self.tracked_files.insert(
            key,
            TrackedFile {
                path: file_path,
                last_modified: metadata.modified()?,
                html: Self::markdown_to_html(&content)?,
            },
        );

        Ok(())
    }

    pub(crate) fn remove_tracked_file(&mut self, key: &str) -> bool {
        self.tracked_files.remove(key).is_some()
    }

    pub(crate) fn markdown_to_html(content: &str) -> Result<String> {
        let mut options = markdown::Options::gfm();
        options.compile.allow_dangerous_html = true;
        options.parse.constructs.frontmatter = true;

        let html_body = markdown::to_html_with_options(content, &options)
            .unwrap_or_else(|_| "Error parsing markdown".to_string());

        Ok(html_body)
    }
}
