# webhook-receiver Specification

## Purpose

Defines the webhook endpoints hooked serves for GitHub and Gitea, how it authenticates and filters incoming push events, and the settings that control this.

## Requirements

### Requirement: GitHub and Gitea endpoints
The system SHALL accept webhook deliveries at `POST /webhooks/github` and `POST /webhooks/gitea`, and SHALL process both with the same validation and build rules.

#### Scenario: Gitea push
- **WHEN** Gitea delivers a correctly signed push event for the watched branch to `/webhooks/gitea`
- **THEN** the system responds 200 with `push event accepted` and builds the pushed commit

#### Scenario: GitHub push
- **WHEN** GitHub delivers a correctly signed push event for the watched branch to `/webhooks/github`
- **THEN** the system responds 200 with `push event accepted` and builds the pushed commit

### Requirement: Signature verification
The system SHALL verify the `X-Hub-Signature-256` header (`sha256=` followed by the hex HMAC-SHA256 of the raw body, keyed with the webhook secret) and SHALL respond 401 when the header is missing or does not match.

#### Scenario: Missing signature
- **WHEN** a request arrives without `X-Hub-Signature-256`
- **THEN** the system responds 401 and does not build

#### Scenario: Wrong secret
- **WHEN** the signature was computed with a different secret
- **THEN** the system responds 401 and does not build

#### Scenario: Malformed signature
- **WHEN** `X-Hub-Signature-256` lacks the `sha256=` prefix or is not valid hex of the right length
- **THEN** the system responds 401 and does not build

### Requirement: Event and branch filtering
The system SHALL read the event type from `X-GitHub-Event`. It SHALL answer `ping` with 200 `ping event ok`, SHALL build only `push` events whose `ref` is `refs/heads/<watched branch>`, and SHALL respond 200 without building for other events, refs, or branches.

#### Scenario: Push to another branch
- **WHEN** a signed push event for a branch other than the watched one arrives
- **THEN** the system responds 200 with `unhandled branch` and does not build

#### Scenario: Tag push
- **WHEN** a signed push event whose `ref` starts with `refs/tags/` arrives
- **THEN** the system responds 200 with `unhandled ref` and does not build

### Requirement: Required push fields
For a push to the watched branch, the system SHALL respond 400 naming the field when any of `repository.full_name`, `after`, or `head_commit.committer.name` is missing or empty. A missing or empty `head_commit.url` SHALL NOT cause a 400; the push is built without a commit URL.

#### Scenario: Missing head commit
- **WHEN** a signed push to the watched branch has no `head_commit`
- **THEN** the system responds 400 and does not build

#### Scenario: Missing head commit URL
- **WHEN** a signed push to the watched branch has `head_commit.committer.name` but no `head_commit.url`
- **THEN** the system responds 200 with `push event accepted` and builds the pushed commit

### Requirement: Provider-neutral settings
The system SHALL take the webhook secret from `--webhook-secret` or `WEBHOOK_SECRET` and the watched branch from `--watch-push-branch` or `WATCH_PUSH_BRANCH`. A webhook secret value starting with `/run/secrets/` SHALL be read from that file with surrounding whitespace trimmed. The system SHALL NOT accept the former `--github-webhook-secret`, `--github-watch-push-branch`, `GITHUB_WEBHOOK_SECRET`, or `GITHUB_WATCH_PUSH_BRANCH`.

#### Scenario: New names
- **WHEN** hooked starts with `--webhook-secret=s --watch-push-branch=main`
- **THEN** it starts and verifies signatures with `s` and watches `main`

#### Scenario: Former flag name
- **WHEN** hooked starts with `--github-webhook-secret=s`
- **THEN** argument parsing fails and hooked exits with an error
