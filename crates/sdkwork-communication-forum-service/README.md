# SDKWork Communication Forum Service

Domain service boundary for `communication/forum`.

This crate owns command validation, orchestration, repository ports, outbox decisions, and domain errors. It does not own HTTP route construction or SQL adapter details.

## Implementation Status

- **Command validation**: Implemented. All service methods validate input fields, enum values, limits, and required fields.
- **Repository ports**: `ForumRepository` trait with 40+ methods. `SqlxForumRepository` provides SQLx-backed implementations for all trait methods.
- **Integration ports**: `ForumDrivePort`, `ForumSearchPort`, `ForumNotificationPort` traits, each receiving the `ForumRequestContext` so the host adapter can scope to the caller's tenant. The host wires the in-process Drive/Search adapters by default (`sdkwork-forum-service-host`, MODULE_SPEC.md §4.2); this crate also ships `NoopForum*Port` (silent success), `LoggingForum*Port` (diagnostic) and the opt-in `HttpForumSearchPort`.
- **Error types**: 12 error variants with HTTP status code mapping.
- **Value objects**: Typed IDs (i64), enums for ModerationStatus, TopicType, Visibility, NodeType, BodyFormat, DataScope.
