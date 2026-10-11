use crate::crypto::verify_signature;
use crate::web::app::Context;
use crate::web::object::base::*;
use crate::web::worker::BuildJob;
use axum::{extract::Extension, http::header::HeaderMap};
use serde_json::Value;
use tracing::info;

pub async fn post_webhooks_github<'a>(
    Extension(ctx): Extension<Context>,
    headers: HeaderMap,
    body: String,
    // Json(payload): Json<serde_json::Value>,
) -> Result<HttpSuccess<'a>, HttpError<'a>> {
    match headers.get("X-Hub-Signature-256") {
        Some(v) => {
            let signature = v.to_str().unwrap_or("");

            match verify_signature(
                ctx.config.github_webhook_secret.as_bytes(),
                body.as_bytes(),
                signature
                    .strip_prefix("sha256=")
                    .unwrap_or_default()
                    .as_bytes(),
            ) {
                Ok(matched) => {
                    if !matched {
                        return Err(HttpError::Unauthorized);
                    }
                }
                Err(e) => {
                    return Err(HttpError::Anyhow(e));
                }
            }
        }
        None => return Err(HttpError::Unauthorized),
    }

    match headers.get("X-GitHub-Event") {
        Some(v) => {
            let event = v.to_str().unwrap_or("");
            match event {
                "ping" => return Ok(HttpSuccess::Success("ping event ok")),
                "push" => {
                    let payload: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
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

                    if branch == ctx.config.github_watch_push_branch {
                        let repo = payload["repository"]["full_name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        if repo.is_empty() {
                            return Err(HttpError::BadRequest("repository.full_name"));
                        }

                        let commit_id = payload["after"].as_str().unwrap_or_default().to_string();
                        if commit_id.is_empty() {
                            return Err(HttpError::BadRequest("after"));
                        }

                        let head_commit_url = payload["head_commit"]["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        if head_commit_url.is_empty() {
                            return Err(HttpError::BadRequest("head_commit.url"));
                        }

                        let head_commit_committer_name = payload["head_commit"]["committer"]
                            ["name"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        if head_commit_committer_name.is_empty() {
                            return Err(HttpError::BadRequest("head_commit.committer.name"));
                        }

                        info!(commit_id = %commit_id, "push event accepted");
                        ctx.build_tx.send_replace(Some(BuildJob {
                            commit_id,
                            repo,
                            branch,
                            head_commit_url,
                            committer: head_commit_committer_name,
                        }));

                        return Ok(HttpSuccess::Success("push event accepted"));
                    }
                    return Ok(HttpSuccess::Success("unhandled branch"));
                }
                _ => return Ok(HttpSuccess::Success("unhandled event")),
            }
        }
        None => return Ok(HttpSuccess::Success("no event")),
    }
}
