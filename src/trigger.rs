use crate::config::Config;
use crate::crypto::sign;
use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::net::IpAddr;

const ACCEPTED: &str = "push event accepted";

/// Sends a signed push event for `commit` to the running server and returns
/// its message.
pub async fn run(config: &Config, commit: &str) -> Result<String> {
    if commit.is_empty() || !commit.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')) {
        bail!("invalid commit ID {commit:?}: expected lowercase hex digits");
    }

    let url = target_url(&config.host, config.port);
    post_push(
        &url,
        &config.webhook_secret,
        &config.watch_push_branch,
        commit,
    )
    .await
}

async fn post_push(url: &str, secret: &str, branch: &str, commit: &str) -> Result<String> {
    let body = json!({
        "ref": format!("refs/heads/{branch}"),
        "after": commit,
        "repository": { "full_name": "-" },
        "head_commit": { "committer": { "name": "manual trigger" } }
    })
    .to_string();

    let res = Client::new()
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-GitHub-Event", "push")
        .header(
            "X-Hub-Signature-256",
            sign(secret.as_bytes(), body.as_bytes())?,
        )
        .body(body)
        .send()
        .await?;

    let status = res.status();
    let text = res.text().await?;
    let message = serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v["result"]["message"].as_str().map(String::from))
        .unwrap_or(text);

    if status.is_success() && message == ACCEPTED {
        Ok(message)
    } else {
        bail!("server answered {status}: {message}")
    }
}

fn target_url(host: &str, port: u16) -> String {
    let host = match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(ip)) if ip.is_unspecified() => "127.0.0.1".to_string(),
        Ok(IpAddr::V6(ip)) if ip.is_unspecified() => "[::1]".to_string(),
        Ok(IpAddr::V6(ip)) => format!("[{ip}]"),
        _ => host.to_string(),
    };
    format!("http://{host}:{port}/webhooks/github")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::app::{new_router, Context};
    use axum::extract::Extension;
    use clap::Parser;
    use std::sync::Arc;
    use tokio::sync::watch;

    const SECRET: &str = "test-secret";
    const BRANCH: &str = "main";
    const COMMIT: &str = "abc123";

    fn config(extra: &[&str]) -> Config {
        let secret_arg = format!("--webhook-secret={SECRET}");
        let branch_arg = format!("--watch-push-branch={BRANCH}");
        let mut args = vec![
            "hooked",
            "--build-entry-script-path=b.sh",
            &secret_arg,
            &branch_arg,
        ];
        args.extend_from_slice(extra);
        Config::try_parse_from(args).unwrap()
    }

    #[tokio::test]
    async fn invalid_commit_is_rejected_before_any_request() {
        for commit in ["", "main", "ABC123", "abc 123", "abc123\n"] {
            let err = run(&config(&[]), commit).await.unwrap_err();
            assert!(err.to_string().contains("invalid commit ID"), "{commit:?}");
        }
    }

    #[test]
    fn target_url_maps_unspecified_hosts_to_loopback() {
        let url = |host| target_url(host, 3000);
        assert_eq!(url("0.0.0.0"), "http://127.0.0.1:3000/webhooks/github");
        assert_eq!(url("::"), "http://[::1]:3000/webhooks/github");
        assert_eq!(url("127.0.0.1"), "http://127.0.0.1:3000/webhooks/github");
        assert_eq!(
            url("2001:db8::1"),
            "http://[2001:db8::1]:3000/webhooks/github"
        );
        assert_eq!(url("hooked"), "http://hooked:3000/webhooks/github");
    }

    async fn serve() -> (
        String,
        watch::Receiver<Option<crate::web::worker::BuildJob>>,
    ) {
        let (tx, rx) = watch::channel(None);
        let ctx = Context {
            config: Arc::new(config(&[])),
            build_tx: Arc::new(tx),
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = target_url("127.0.0.1", listener.local_addr().unwrap().port());
        tokio::spawn(async move {
            axum::serve(listener, new_router().layer(Extension(ctx)))
                .await
                .unwrap();
        });
        (url, rx)
    }

    #[tokio::test]
    async fn trigger_is_accepted_and_enqueues_a_job() {
        let (url, rx) = serve().await;
        let message = post_push(&url, SECRET, BRANCH, COMMIT).await.unwrap();
        assert_eq!(message, ACCEPTED);

        let job = rx.borrow().clone().unwrap();
        assert_eq!(job.commit_id, COMMIT);
        assert_eq!(job.branch, BRANCH);
        assert_eq!(job.head_commit_url, None);
    }

    #[tokio::test]
    async fn wrong_secret_is_an_error() {
        let (url, rx) = serve().await;
        let err = post_push(&url, "other-secret", BRANCH, COMMIT)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("401"), "{err}");
        assert!(rx.borrow().is_none());
    }

    #[tokio::test]
    async fn other_branch_is_an_error() {
        let (url, rx) = serve().await;
        let err = post_push(&url, SECRET, "dev", COMMIT).await.unwrap_err();
        assert!(err.to_string().contains("unhandled branch"), "{err}");
        assert!(rx.borrow().is_none());
    }

    #[tokio::test]
    async fn closed_port_is_an_error() {
        // Port 1 on loopback refuses connections.
        let url = target_url("127.0.0.1", 1);
        assert!(post_push(&url, SECRET, BRANCH, COMMIT).await.is_err());
    }
}
