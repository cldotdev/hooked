## Purpose

Defines how hooked reports the outcome of each build, through a Discord webhook when one is configured and through the log file otherwise.

## ADDED Requirements

### Requirement: Discord notification is optional
The system SHALL run without a Discord webhook URL configured. The URL is configured with `--discord-webhook-url` or `DISCORD_WEBHOOK_URL`; an empty value SHALL be treated as unset.

#### Scenario: No Discord URL
- **WHEN** hooked starts with neither `--discord-webhook-url` nor `DISCORD_WEBHOOK_URL` set
- **THEN** it starts and serves webhooks normally

#### Scenario: Empty environment variable
- **WHEN** hooked starts with `DISCORD_WEBHOOK_URL` set to an empty string
- **THEN** it behaves as if no Discord URL were configured

### Requirement: Discord message for each build
When a Discord webhook URL is configured, the system SHALL send one message per finished build. The message SHALL contain an embed titled `Deployment Success` or `Deployment Failed` according to the script's exit status, linking to the head commit and listing the repository, branch, short commit ID, and committer, and SHALL attach the script's stdout and stderr as `stdout.txt` and `stderr.txt`.

#### Scenario: Successful build
- **WHEN** the build script exits with status 0 and Discord is configured
- **THEN** Discord receives an embed titled `Deployment Success` with stdout and stderr attached

#### Scenario: Failed build
- **WHEN** the build script exits non-zero or cannot be started, and Discord is configured
- **THEN** Discord receives an embed titled `Deployment Failed` with stdout and stderr attached

### Requirement: Build outcome is always logged
The system SHALL write each build's commit ID and exit status to the log, whether or not Discord is configured, and SHALL NOT write the script's stdout or stderr to the log.

#### Scenario: Discord not configured
- **WHEN** a build finishes and no Discord URL is configured
- **THEN** the log contains the commit ID and exit status, and no Discord request is sent

### Requirement: Notification failure does not affect builds
The system SHALL log a failed Discord request and continue processing later pushes.

#### Scenario: Discord unreachable
- **WHEN** the Discord request fails
- **THEN** the failure is logged and the next accepted push is still built
