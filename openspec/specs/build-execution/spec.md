# build-execution Specification

## Purpose

Defines how hooked turns accepted push events into runs of the build entry script, including ordering and which pushes are skipped.

## Requirements

### Requirement: Accepted pushes are acknowledged before the build runs
The system SHALL respond to an accepted push event with HTTP 200 and the message `push event accepted` without waiting for the build script to finish.

#### Scenario: Long-running build
- **WHEN** a valid push to the watched branch arrives and the build script takes several minutes
- **THEN** the webhook response returns immediately with status 200 and message `push event accepted`

### Requirement: Builds run one at a time
The system SHALL never run two instances of the build entry script at the same time.

#### Scenario: Push arrives during a build
- **WHEN** a push is accepted while a build for an earlier push is still running
- **THEN** the new build starts only after the running build has exited

### Requirement: Only the newest pending push is built
While a build is running, the system SHALL keep only the most recently accepted push as pending and SHALL discard any push it replaces. When the running build exits, the system SHALL start the pending push, if any.

#### Scenario: Several pushes arrive during a build
- **WHEN** pushes A, B, and C are accepted in that order while a build for an earlier push is running
- **THEN** after that build exits, the system builds C only, and A and B are never built

#### Scenario: Idle worker
- **WHEN** a push is accepted while no build is running
- **THEN** the system starts its build without waiting for another push

### Requirement: The script receives the commit ID
The system SHALL run the build entry script with the pushed commit ID (`after`) as its first argument.

#### Scenario: Script invocation
- **WHEN** the system builds a push whose `after` field is `abc1234...`
- **THEN** the build entry script runs with `abc1234...` as its first argument

### Requirement: A failing build does not stop later builds
The system SHALL keep processing later pushes when the build script cannot be started, exits non-zero, or produces output that is not valid UTF-8.

#### Scenario: Script path missing
- **WHEN** the build entry script cannot be executed for one push
- **THEN** the system records the failure and still builds the next accepted push
