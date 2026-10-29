## MODIFIED Requirements

### Requirement: Discord message for each build
When a Discord webhook URL is configured, the system SHALL send one message per finished build. The message SHALL contain an embed titled `Deployment Success` or `Deployment Failed` according to the script's exit status, listing the repository, branch, short commit ID, and committer, and SHALL attach the script's stdout and stderr as `stdout.txt` and `stderr.txt`. The embed SHALL link to the head commit when the push carried `head_commit.url`, and SHALL have no link otherwise.

#### Scenario: Successful build
- **WHEN** the build script exits with status 0 and Discord is configured
- **THEN** Discord receives an embed titled `Deployment Success` with stdout and stderr attached

#### Scenario: Failed build
- **WHEN** the build script exits non-zero or cannot be started, and Discord is configured
- **THEN** Discord receives an embed titled `Deployment Failed` with stdout and stderr attached

#### Scenario: Push without a commit URL
- **WHEN** a build for a push without `head_commit.url` finishes and Discord is configured
- **THEN** Discord receives the embed without a `url` field
