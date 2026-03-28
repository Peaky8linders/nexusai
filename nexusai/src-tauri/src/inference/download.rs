//! Model download manager — downloads GGUF models from HuggingFace with
//! progress reporting and resume support.

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Download progress state
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub model_id: String,
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub percent: f64,
    pub speed_mbps: f64,
    pub eta_seconds: f64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Complete,
    Failed(String),
    Paused,
}

/// HuggingFace model file metadata (used for API responses)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HfFileInfo {
    size: u64,
    #[serde(rename = "lfs")]
    lfs_info: Option<HfLfsInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HfLfsInfo {
    size: u64,
    sha256: Option<String>,
}

/// Download a GGUF model from HuggingFace
pub async fn download_model(
    repo: &str,
    filename: &str,
    dest_dir: &Path,
    progress_callback: impl Fn(DownloadProgress) + Send + 'static,
) -> Result<PathBuf, String> {
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        repo, filename
    );
    let dest_path = dest_dir.join(filename);

    // Check for existing partial download (resume support)
    let existing_size = if dest_path.exists() {
        tokio::fs::metadata(&dest_path)
            .await
            .map(|m| m.len())
            .unwrap_or(0)
    } else {
        0
    };

    let client = reqwest::Client::new();

    // Get file size via HEAD request
    let head_resp = client
        .head(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to check file size: {}", e))?;

    let total_size = head_resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    if existing_size == total_size && total_size > 0 {
        progress_callback(DownloadProgress {
            model_id: repo.to_string(),
            file_name: filename.to_string(),
            bytes_downloaded: total_size,
            bytes_total: total_size,
            percent: 100.0,
            speed_mbps: 0.0,
            eta_seconds: 0.0,
            status: DownloadStatus::Complete,
        });
        return Ok(dest_path);
    }

    // Build request with Range header for resume
    let mut req = client.get(&url);
    if existing_size > 0 {
        req = req.header(reqwest::header::RANGE, format!("bytes={}-", existing_size));
        tracing::info!(
            "Resuming download from {} bytes ({:.1}%)",
            existing_size,
            existing_size as f64 / total_size as f64 * 100.0
        );
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(format!("Download failed with status: {}", resp.status()));
    }

    // Open file for writing (append if resuming)
    let mut file = if existing_size > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&dest_path)
            .await
            .map_err(|e| format!("Failed to open file for resume: {}", e))?
    } else {
        tokio::fs::File::create(&dest_path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?
    };

    let mut downloaded = existing_size;
    let start_time = std::time::Instant::now();
    let mut last_report = std::time::Instant::now();
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Write error: {}", e))?;

        downloaded += chunk.len() as u64;

        // Report progress every 500ms
        if last_report.elapsed().as_millis() > 500 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = (downloaded - existing_size) as f64 / elapsed / 1_000_000.0;
            let remaining = total_size.saturating_sub(downloaded) as f64;
            let eta = if speed > 0.0 {
                remaining / (speed * 1_000_000.0)
            } else {
                0.0
            };

            progress_callback(DownloadProgress {
                model_id: repo.to_string(),
                file_name: filename.to_string(),
                bytes_downloaded: downloaded,
                bytes_total: total_size,
                percent: downloaded as f64 / total_size.max(1) as f64 * 100.0,
                speed_mbps: speed * 8.0,
                eta_seconds: eta,
                status: DownloadStatus::Downloading,
            });
            last_report = std::time::Instant::now();
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("Flush error: {}", e))?;

    // Signal complete
    progress_callback(DownloadProgress {
        model_id: repo.to_string(),
        file_name: filename.to_string(),
        bytes_downloaded: downloaded,
        bytes_total: total_size,
        percent: 100.0,
        speed_mbps: 0.0,
        eta_seconds: 0.0,
        status: DownloadStatus::Complete,
    });

    tracing::info!("Download complete: {} ({:.1} GB)", filename, downloaded as f64 / 1e9);
    Ok(dest_path)
}

/// Verify a downloaded file against an expected SHA256 hash
pub async fn verify_sha256(path: &Path, expected: &str) -> Result<bool, String> {
    let data = tokio::fs::read(path)
        .await
        .map_err(|e| format!("Failed to read file for verification: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = hex::encode(hasher.finalize());

    Ok(hash == expected)
}

/// Scan a directory for .gguf files and return their paths
pub async fn scan_models_dir(dir: &Path) -> Vec<PathBuf> {
    let mut models = Vec::new();
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("gguf") {
                models.push(path);
            }
        }
    }
    models
}
