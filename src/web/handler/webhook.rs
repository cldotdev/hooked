use crate::crypto::verify_signature;
use crate::web::app::Context;
use crate::web::object::base::*;
use crate::web::worker::BuildJob;
use axum::{extract::Extension, http::header::HeaderMap};
use serde_json::Value;
use tracing::info;

/// Serves both `/webhooks/github` and `/webhooks/gitea`: Gitea sends the
/// GitHub-compatible `X-GitHub-Event` and `X-Hub-Signature-256` headers.
pub async fn post_webhooks<'a>(
    Extension(ctx): Extension<Context>,
    headers: HeaderMap,
    body: String,
) -> Result<HttpSuccess<'a>, HttpError<'a>> {
    let signature = headers
        .get("X-Hub-Signature-256")
        .ok_or(HttpError::Unauthorized)?
        .to_str()
        .unwrap_or("");

    // A malformed signature (bad prefix or hex) is a mismatch, not a server error.
    let matched = verify_signature(
        ctx.config.webhook_secret.as_bytes(),
        body.as_bytes(),
        signature
            .strip_prefix("sha256=")
            .unwrap_or_default()
            .as_bytes(),
    )
    .unwrap_or(false);
    if !matched {
        return Err(HttpError::Unauthorized);
    }

    let Some(event) = headers.get("X-GitHub-Event") else {
        return Ok(HttpSuccess::Success("no event"));
    };

    match event.to_str().unwrap_or("") {
        "ping" => Ok(HttpSuccess::Success("ping event ok")),
        "push" => handle_push(&ctx, &body),
        _ => Ok(HttpSuccess::Success("unhandled event")),
    }
}

fn handle_push<'a>(ctx: &Context, body: &str) -> Result<HttpSuccess<'a>, HttpError<'a>> {
    let payload: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    if payload == Value::Null {
        return Err(HttpError::BadRequest("payload"));
    }

    let branch = payload["ref"]
        .as_str()
        .unwrap_or_default()
        .strip_prefix("refs/heads/")
        .unwrap_or_default()
        .to_string();

    if branch.is_empty() {
        return Ok(HttpSuccess::Success("unhandled ref"));
    }

    if branch != ctx.config.watch_push_branch {
        return Ok(HttpSuccess::Success("unhandled branch"));
    }

    let repo = required(&payload["repository"]["full_name"], "repository.full_name")?;
    let commit_id = required(&payload["after"], "after")?;
    let head_commit_url = required(&payload["head_commit"]["url"], "head_commit.url")?;
    let committer = required(
        &payload["head_commit"]["committer"]["name"],
        "head_commit.committer.name",
    )?;

    info!(commit_id = %commit_id, "push event accepted");
    ctx.build_tx.send_replace(Some(BuildJob {
        commit_id,
        repo,
        branch,
        head_commit_url,
        committer,
    }));

    Ok(HttpSuccess::Success("push event accepted"))
}

fn required<'a>(value: &Value, name: &'static str) -> Result<String, HttpError<'a>> {
    match value.as_str() {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(HttpError::BadRequest(name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::web::app::new_router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use clap::Parser;
    use hmac::{Hmac, KeyInit, Mac};
    use serde_json::json;
    use sha2::Sha256;
    use std::sync::Arc;
    use tokio::sync::watch;
    use tower::ServiceExt;

    const SECRET: &str = "test-secret";
    const BRANCH: &str = "main";

    struct Response {
        status: StatusCode,
        message: String,
    }

    fn sign(secret: &str, body: &str) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    fn github_push() -> Value {
        json!({
            "ref": "refs/heads/main",
            "after": "1111111111111111111111111111111111111111",
            "repository": { "full_name": "octocat/hello-world" },
            "head_commit": {
                "url": "https://github.example.com/octocat/hello-world/commit/1111111",
                "committer": { "name": "Mona Lisa" }
            }
        })
    }

    fn gitea_push() -> Value {
        json!({
            "ref": "refs/heads/main",
            "before": "0000000000000000000000000000000000000000",
            "after": "2222222222222222222222222222222222222222",
            "repository": { "full_name": "alice/demo", "html_url": "https://git.example.com/alice/demo" },
            "pusher": { "login": "alice" },
            "head_commit": {
                "id": "2222222222222222222222222222222222222222",
                "url": "https://git.example.com/alice/demo/commit/2222222",
                "committer": { "name": "Alice Example", "email": "alice@example.com" }
            }
        })
    }

    /// Sends one request through the router and returns the response plus the queued job.
    async fn send(
        path: &str,
        event: Option<&str>,
        signature: Option<String>,
        body: &str,
    ) -> (Response, Option<BuildJob>) {
        let config = Config::try_parse_from([
            "hooked",
            "--build-entry-script-path=b.sh",
            &format!("--webhook-secret={SECRET}"),
            &format!("--watch-push-branch={BRANCH}"),
        ])
        .unwrap();
        let (tx, rx) = watch::channel(None);
        let ctx = Context {
            config: Arc::new(config),
            build_tx: Arc::new(tx),
        };

        let mut req = Request::post(path);
        if let Some(event) = event {
            req = req.header("X-GitHub-Event", event);
        }
        if let Some(signature) = signature {
            req = req.header("X-Hub-Signature-256", signature);
        }
        let res = new_router()
            .layer(Extension(ctx))
            .oneshot(req.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();

        let status = res.status();
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&bytes).unwrap();
        let message = json["result"]["message"].as_str().unwrap().to_string();
        let job = rx.borrow().clone();
        (Response { status, message }, job)
    }

    async fn send_signed(path: &str, event: &str, payload: &Value) -> (Response, Option<BuildJob>) {
        let body = payload.to_string();
        send(path, Some(event), Some(sign(SECRET, &body)), &body).await
    }

    #[tokio::test]
    async fn gitea_push_is_accepted_and_enqueued() {
        let (res, job) = send_signed("/webhooks/gitea", "push", &gitea_push()).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.message, "push event accepted");
        let job = job.unwrap();
        assert_eq!(job.commit_id, "2222222222222222222222222222222222222222");
        assert_eq!(job.repo, "alice/demo");
        assert_eq!(job.branch, BRANCH);
        assert_eq!(job.committer, "Alice Example");
    }

    #[tokio::test]
    async fn github_push_is_accepted_and_enqueued() {
        let (res, job) = send_signed("/webhooks/github", "push", &github_push()).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.message, "push event accepted");
        assert_eq!(job.unwrap().repo, "octocat/hello-world");
    }

    #[tokio::test]
    async fn missing_signature_is_unauthorized() {
        let body = github_push().to_string();
        let (res, job) = send("/webhooks/gitea", Some("push"), None, &body).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert!(job.is_none());
    }

    #[tokio::test]
    async fn wrong_secret_is_unauthorized() {
        let body = github_push().to_string();
        let sig = sign("other-secret", &body);
        let (res, job) = send("/webhooks/github", Some("push"), Some(sig), &body).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert!(job.is_none());
    }

    #[tokio::test]
    async fn malformed_signature_is_unauthorized() {
        let body = github_push().to_string();
        let (res, _) = send("/webhooks/github", Some("push"), Some("nope".into()), &body).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn other_branch_is_unhandled() {
        let mut payload = gitea_push();
        payload["ref"] = json!("refs/heads/dev");
        let (res, job) = send_signed("/webhooks/gitea", "push", &payload).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.message, "unhandled branch");
        assert!(job.is_none());
    }

    #[tokio::test]
    async fn tag_ref_is_unhandled() {
        let mut payload = gitea_push();
        payload["ref"] = json!("refs/tags/v1.0.0");
        let (res, job) = send_signed("/webhooks/gitea", "push", &payload).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.message, "unhandled ref");
        assert!(job.is_none());
    }

    #[tokio::test]
    async fn push_without_head_commit_is_bad_request() {
        let mut payload = gitea_push();
        payload.as_object_mut().unwrap().remove("head_commit");
        let (res, job) = send_signed("/webhooks/gitea", "push", &payload).await;
        assert_eq!(res.status, StatusCode::BAD_REQUEST);
        assert!(res.message.contains("head_commit.url"));
        assert!(job.is_none());
    }

    #[tokio::test]
    async fn ping_is_acknowledged() {
        let (res, job) = send_signed("/webhooks/gitea", "ping", &json!({ "zen": "ok" })).await;
        assert_eq!(res.status, StatusCode::OK);
        assert_eq!(res.message, "ping event ok");
        assert!(job.is_none());
    }
}
