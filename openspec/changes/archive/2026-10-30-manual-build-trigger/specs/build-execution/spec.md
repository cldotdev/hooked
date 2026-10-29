## ADDED Requirements

### Requirement: Manual trigger through the running server
`hooked trigger <commit>` SHALL send a push event for `refs/heads/<watched branch>` with `<commit>` as `after`, signed with the configured webhook secret, to `/webhooks/github` on the running server at the configured host and port, connecting to loopback when the host is an unspecified address (`0.0.0.0` or `::`). It SHALL read the webhook secret, watched branch, host, and port from the same flags and environment variables as the server. It SHALL exit 0 when the server answers 200 with `push event accepted`, and otherwise print the server's answer or the connection error and exit non-zero. It SHALL reject a commit that is not a non-empty string of lowercase hex digits without sending a request. `hooked` without a subcommand SHALL start the server.

#### Scenario: Trigger on an idle server
- **WHEN** no build is running and `hooked trigger <commit>` runs against the server
- **THEN** the command exits 0 and the server runs the build entry script with `<commit>` as its first argument

#### Scenario: Trigger during a build
- **WHEN** `hooked trigger <commit>` runs while a build is running
- **THEN** the trigger replaces any pending push, and its build starts only after the running build exits

#### Scenario: Server not running
- **WHEN** `hooked trigger <commit>` runs and nothing listens on the configured port
- **THEN** the command prints the connection error and exits non-zero

#### Scenario: Invalid commit
- **WHEN** `hooked trigger main` runs
- **THEN** the command prints an error, sends no request, and exits non-zero
