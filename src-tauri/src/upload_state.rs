use serde::Serialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UploadSnapshot {
    pub id: u64,
    pub file_name: String,
    pub status: String,
    pub url: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedUpload {
    pub id: u64,
    pub file_path: String,
    pub file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UploadStatus {
    Uploading,
    Done,
    Error,
}

impl UploadStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Uploading => "uploading",
            Self::Done => "done",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
struct UploadEntry {
    id: u64,
    file_name: String,
    status: UploadStatus,
    url: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Default)]
pub struct UploadTracker {
    next_id: AtomicU64,
    uploads: Mutex<Vec<UploadEntry>>,
}

impl UploadTracker {
    pub fn begin_batch(&self, files: &[String]) -> Vec<QueuedUpload> {
        let mut uploads = self.uploads.lock().expect("upload tracker poisoned");
        let mut queued = Vec::with_capacity(files.len());

        for file_path in files {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
            let file_name = extract_file_name(file_path);

            uploads.push(UploadEntry {
                id,
                file_name: file_name.clone(),
                status: UploadStatus::Uploading,
                url: None,
                message: None,
            });

            queued.push(QueuedUpload {
                id,
                file_path: file_path.clone(),
                file_name,
            });
        }

        queued
    }

    pub fn mark_done(&self, id: u64, url: String) {
        self.update_entry(id, UploadStatus::Done, Some(url), None);
    }

    pub fn mark_error(&self, id: u64, message: String) {
        self.update_entry(id, UploadStatus::Error, None, Some(message));
    }

    pub fn snapshot(&self) -> Vec<UploadSnapshot> {
        let uploads = self.uploads.lock().expect("upload tracker poisoned");

        uploads
            .iter()
            .map(|entry| UploadSnapshot {
                id: entry.id,
                file_name: entry.file_name.clone(),
                status: entry.status.as_str().to_string(),
                url: entry.url.clone(),
                message: entry.message.clone(),
            })
            .collect()
    }

    pub fn clear_finished(&self, ids: &[u64]) {
        let ids_to_clear: HashSet<u64> = ids.iter().copied().collect();
        let mut uploads = self.uploads.lock().expect("upload tracker poisoned");

        uploads.retain(|entry| {
            !(ids_to_clear.contains(&entry.id) && entry.status != UploadStatus::Uploading)
        });
    }

    fn update_entry(
        &self,
        id: u64,
        status: UploadStatus,
        url: Option<String>,
        message: Option<String>,
    ) {
        let mut uploads = self.uploads.lock().expect("upload tracker poisoned");

        if let Some(entry) = uploads.iter_mut().find(|entry| entry.id == id) {
            entry.status = status;
            entry.url = url;
            entry.message = message;
        }
    }
}

fn extract_file_name(file_path: &str) -> String {
    Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file_path)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::UploadTracker;

    #[test]
    fn begin_batch_creates_uploading_entries_with_file_names() {
        let tracker = UploadTracker::default();
        let queued = tracker.begin_batch(&[
            String::from(r"C:\Users\alice\Desktop\report.pdf"),
            String::from(r"D:\images\photo.png"),
        ]);

        assert_eq!(queued.len(), 2);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot[0].file_name, "report.pdf");
        assert_eq!(snapshot[0].status, "uploading");
        assert_eq!(snapshot[1].file_name, "photo.png");
        assert_eq!(snapshot[1].status, "uploading");
    }

    #[test]
    fn completing_uploads_updates_status_and_payload() {
        let tracker = UploadTracker::default();
        let queued = tracker.begin_batch(&[String::from(r"C:\tmp\archive.zip")]);

        tracker.mark_done(queued[0].id, String::from("https://example.com/archive.zip"));

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot[0].status, "done");
        assert_eq!(
            snapshot[0].url.as_deref(),
            Some("https://example.com/archive.zip")
        );
        assert_eq!(snapshot[0].message, None);
    }

    #[test]
    fn clear_finished_keeps_active_uploads() {
        let tracker = UploadTracker::default();
        let queued = tracker.begin_batch(&[
            String::from(r"C:\tmp\done.txt"),
            String::from(r"C:\tmp\still-uploading.txt"),
        ]);

        tracker.mark_error(queued[0].id, String::from("network error"));
        tracker.clear_finished(&[queued[0].id, queued[1].id]);

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].id, queued[1].id);
        assert_eq!(snapshot[0].status, "uploading");
    }
}
