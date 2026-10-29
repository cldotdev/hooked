## MODIFIED Requirements

### Requirement: Required push fields
For a push to the watched branch, the system SHALL respond 400 naming the field when any of `repository.full_name`, `after`, or `head_commit.committer.name` is missing or empty. A missing or empty `head_commit.url` SHALL NOT cause a 400; the push is built without a commit URL.

#### Scenario: Missing head commit
- **WHEN** a signed push to the watched branch has no `head_commit`
- **THEN** the system responds 400 and does not build

#### Scenario: Missing head commit URL
- **WHEN** a signed push to the watched branch has `head_commit.committer.name` but no `head_commit.url`
- **THEN** the system responds 200 with `push event accepted` and builds the pushed commit
