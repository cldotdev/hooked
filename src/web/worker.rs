use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::{json, Value};
use std::process::Output;
use tokio::process::Command;
use tokio::sync::watch;
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct BuildJob {
    pub commit_id: String,
    pub repo: String,
    pub branch: String,
    pub head_commit_url: Option<String>,
    pub committer: String,
}

pub type JobSender = watch::Sender<Option<BuildJob>>;

struct BuildResult {
    success: bool,
    status: String,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Starts the single build worker and returns the sender used to enqueue jobs.
///
/// A watch channel keeps only the latest value, so jobs sent while a build
/// runs overwrite each other and only the newest one is built next.
pub fn spawn_worker(script_path: String, discord_webhook_url: Option<String>) -> JobSender {
    let (tx, mut rx) = watch::channel::<Option<BuildJob>>(None);

    tokio::spawn(async move {
        let client = Client::new();
        while rx.changed().await.is_ok() {
            let Some(job) = rx.borrow_and_update().clone() else {
                continue;
            };

            let result = run_build(&script_path, &job.commit_id).await;
            if result.success {
                info!(commit_id = %job.commit_id, status = %result.status, "build finished");
            } else {
                warn!(commit_id = %job.commit_id, status = %result.status, "build failed");
            }

            if let Some(url) = &discord_webhook_url {
                if let Err(e) = notify_discord(&client, url, &job, result).await {
                    error!(commit_id = %job.commit_id, "failed to notify Discord: {}", e);
                }
            }
        }
    });

    tx
}

async fn run_build(script_path: &str, commit_id: &str) -> BuildResult {
    match Command::new(script_path).arg(commit_id).output().await {
        Ok(Output {
            status,
            stdout,
            stderr,
        }) => BuildResult {
            success: status.success(),
            status: status.to_string(),
            stdout,
            stderr,
        },
        Err(e) => BuildResult {
            success: false,
            status: "failed to start".to_string(),
            stdout: Vec::new(),
            stderr: e.to_string().into_bytes(),
        },
    }
}

async fn notify_discord(
    client: &Client,
    url: &str,
    job: &BuildJob,
    result: BuildResult,
) -> reqwest::Result<()> {
    let form = Form::new()
        .text(
            "payload_json",
            embed_payload(job, result.success).to_string(),
        )
        .part("file1", Part::bytes(result.stdout).file_name("stdout.txt"))
        .part("file2", Part::bytes(result.stderr).file_name("stderr.txt"));

    client
        .post(url)
        .multipart(form)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

fn embed_payload(job: &BuildJob, success: bool) -> Value {
    let (title, color) = if success {
        ("Deployment Success", 10731148) // #a3be8c
    } else {
        ("Deployment Failed", 12542314) // #bf616a
    };
    let short_commit_id: String = job.commit_id.chars().take(7).collect();

    let mut embed = json!({
        "title": title,
        "color": color,
        "fields": [
            { "name": "Repository", "value": job.repo },
            { "name": "Branch", "value": job.branch, "inline": true },
            { "name": "Commit ID", "value": short_commit_id, "inline": true },
            { "name": "Committer", "value": job.committer }
        ]
    });
    if let Some(url) = &job.head_commit_url {
        embed["url"] = json!(url);
    }

    json!({ "embeds": [embed] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::Duration;

    fn job(commit_id: &str) -> BuildJob {
        BuildJob {
            commit_id: commit_id.to_string(),
            repo: "owner/repo".to_string(),
            branch: "master".to_string(),
            head_commit_url: Some("https://example.com/commit".to_string()),
            committer: "Alice".to_string(),
        }
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hooked-{}-{}", name, std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_script(dir: &std::path::Path, body: &str) -> PathBuf {
        let path = dir.join("build.sh");
        std::fs::write(&path, format!("#!/bin/sh\n{}\n", body)).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    async fn wait_for_lines(log: &std::path::Path, n: usize) -> Vec<String> {
        for _ in 0..200 {
            let lines: Vec<String> = std::fs::read_to_string(log)
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect();
            if lines.len() >= n {
                return lines;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("timed out waiting for {} log lines", n);
    }

    #[tokio::test]
    async fn only_first_and_newest_pending_job_are_built() {
        let dir = test_dir("coalesce");
        let log = dir.join("log");
        let script = write_script(
            &dir,
            &format!("echo \"$1\" >> {}\nsleep 0.5", log.display()),
        );
        let tx = spawn_worker(script.to_str().unwrap().to_string(), None);

        tx.send_replace(Some(job("a")));
        wait_for_lines(&log, 1).await;
        for id in ["b", "c", "d"] {
            tx.send_replace(Some(job(id)));
        }

        wait_for_lines(&log, 2).await;
        // Leave room for an unexpected extra build to show up.
        tokio::time::sleep(Duration::from_millis(1200)).await;
        assert_eq!(wait_for_lines(&log, 2).await, ["a", "d"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn missing_script_is_a_failed_build_and_next_job_runs() {
        let result = run_build("/nonexistent/build.sh", "abc").await;
        assert!(!result.success);
        assert!(!result.stderr.is_empty());

        let dir = test_dir("missing");
        let log = dir.join("log");
        let script = dir.join("build.sh");
        let tx = spawn_worker(script.to_str().unwrap().to_string(), None);

        // The script does not exist yet, so this build fails to start.
        tx.send_replace(Some(job("a")));
        tokio::time::sleep(Duration::from_millis(200)).await;

        write_script(&dir, &format!("echo \"$1\" >> {}", log.display()));
        tx.send_replace(Some(job("b")));
        assert_eq!(wait_for_lines(&log, 1).await, ["b"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn discord_failure_does_not_stop_later_builds() {
        let dir = test_dir("discord");
        let log = dir.join("log");
        let script = write_script(&dir, &format!("echo \"$1\" >> {}", log.display()));
        // Port 1 on loopback refuses connections, so every notification fails.
        let tx = spawn_worker(
            script.to_str().unwrap().to_string(),
            Some("http://127.0.0.1:1/".to_string()),
        );

        tx.send_replace(Some(job("a")));
        wait_for_lines(&log, 1).await;
        tx.send_replace(Some(job("b")));
        assert_eq!(wait_for_lines(&log, 2).await, ["a", "b"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn non_utf8_output_is_accepted() {
        let dir = test_dir("utf8");
        let script = write_script(&dir, "printf '\\377\\376'; exit 3");
        let result = run_build(script.to_str().unwrap(), "abc").await;
        assert!(!result.success);
        assert_eq!(result.stdout, [0xff, 0xfe]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn embed_payload_for_success() {
        let v = embed_payload(&job("0123456789"), true);
        let e = &v["embeds"][0];
        assert_eq!(e["title"], "Deployment Success");
        assert_eq!(e["color"], 10731148);
        assert_eq!(e["url"], "https://example.com/commit");
        assert_eq!(e["fields"][0]["value"], "owner/repo");
        assert_eq!(e["fields"][1]["value"], "master");
        assert_eq!(e["fields"][2]["value"], "0123456");
        assert_eq!(e["fields"][3]["value"], "Alice");
    }

    #[test]
    fn embed_payload_without_url_has_no_url_key() {
        let mut job = job("abc");
        job.head_commit_url = None;
        let v = embed_payload(&job, true);
        assert!(v["embeds"][0].get("url").is_none());
    }

    #[test]
    fn embed_payload_for_failure_with_short_commit_id() {
        let v = embed_payload(&job("abc"), false);
        let e = &v["embeds"][0];
        assert_eq!(e["title"], "Deployment Failed");
        assert_eq!(e["color"], 12542314);
        assert_eq!(e["fields"][2]["value"], "abc");
    }
}
