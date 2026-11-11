# Forum Integration Roadmap

## Foundation Dependencies

Current metadata avoids declaring unresolved SDK dependencies as generation inputs. The following integrations must be added once their local SDK family paths and authority contracts are verified:

- Appbase app SDK: login/session/current user/workspace context for app clients.
- Appbase backend SDK: backend-admin IAM and permission management.
- Drive app/backend SDKs: attachment upload sessions, media resource selection, and download grants.
- Search backend SDK: full reindex and incremental indexing adapters.
- Messaging or notification SDK: subscription and moderation notification delivery.

## Integration Status

### SDK Dependencies

Planned `sdkDependencies` entries for each SDK family:

```yaml
# sdkwork-forum-app-sdk
sdkDependencies:
  - sdkFamily: sdkwork-iam-app-sdk
    authority: sdkwork-iam-app-api
    purpose: IAM login, session, current user context
  - sdkFamily: sdkwork-drive-app-sdk
    authority: sdkwork-drive-app-api
    purpose: Attachment upload sessions and media resource selection

# sdkwork-forum-backend-sdk
sdkDependencies:
  - sdkFamily: sdkwork-iam-backend-sdk
    authority: sdkwork-iam-backend-api
    purpose: IAM permission management and operator context
  - sdkFamily: sdkwork-drive-backend-sdk
    authority: sdkwork-drive-backend-api
    purpose: Download grants and media lifecycle management
  - sdkFamily: sdkwork-search-backend-sdk
    authority: sdkwork-search-backend-api
    purpose: Full reindex and incremental indexing

# sdkwork-forum-sdk (open)
sdkDependencies: []
# Open API is anonymous public reads only; no external SDK dependencies required.
```

### Service Ports (Implemented)

| Port | Trait | Status | Implementations |
|------|-------|--------|-----------------|
| Drive | `ForumDrivePort` | Implemented (in-process) | `InProcessForumDrivePort` (`sdkwork-drive-workspace-service` on the shared pool), `LoggingForumDrivePort`, `NoopForumDrivePort` |
| Search | `ForumSearchPort` | Implemented (in-process by default) | `InProcessForumSearchPort` (`sdkwork-search` provider SPI), `HttpForumSearchPort` (explicit `SDKWORK_FORUM_SEARCH_MODE=http` opt-in), `LoggingForumSearchPort`, `NoopForumSearchPort` |
| Notification | `ForumNotificationPort` | Partial | `HttpForumNotificationPort` (generic HTTP), `LoggingForumNotificationPort`, `NoopForumNotificationPort` |

### IAM Request Context

When `SDKWORK_FORUM_IAM_ENABLED=true`, the forum API server resolves `Authorization` + `Access-Token` against `iam_session` via `sdkwork-iam-web-adapter`. The resolved tenant/org/user ids populate `ForumRequestContext` before handlers run. Set `SDKWORK_FORUM_IAM_STRICT=true` to reject invalid sessions on app/backend forum routes instead of falling back to header/env defaults.

Use `SDKWORK_DATABASE_URL` when IAM sessions live outside the forum database module; otherwise the forum PostgreSQL pool is reused.

### Drive Media Grants

`ForumDrivePort` provides (called in-process through `sdkwork-drive-workspace-service`, MODULE_SPEC.md §4.2):
- `validate_media_reference(ctx, media_resource_id)` - Drive's tenant-scoped active node + active space read, plus the caller's own `reader` effective ACL
- `create_download_grant(ctx, media_resource_id)` - Drive download-token URL (Drive's 30..=300 s window, default 120 s) on this process's public ingress

There is no Drive HTTP transport: the retired `SDKWORK_FORUM_DRIVE_URL` only selected a logging no-op and has been deleted.

### Notification Event Publisher

`ForumNotificationPort` provides:
- `publish_forum_event(event_type, aggregate_id)` - Generic forum event publication
- `publish_moderation_alert(case_id, severity)` - Moderation alert to operators
- `publish_subscription_notification(user_id, event_type, target_id)` - Subscription delivery

Awaiting `sdkwork-messaging-sdk` dependency resolution for real implementation.

### Search Indexing Adapter

`ForumSearchPort` provides (in-process through the `sdkwork-search` provider SPI by default):
- `index_document(ctx, source_type, source_id)` - Upsert document `{sourceType}:{sourceId}` into index `SDKWORK_FORUM_SEARCH_INDEX_ID` (default `forum`) under the request tenant/organization
- `delete_document(ctx, source_type, source_id)` - Soft-delete the same document
- `rebuild_index(ctx, board_id)` - Drop + recreate the index; board-scoped rebuild remains a forum-side concern

A split deployment where sdkwork-search runs in another process is an explicit opt-in: `SDKWORK_FORUM_SEARCH_MODE=http` together with `SDKWORK_FORUM_SEARCH_URL` and a private bootstrap `SDKWORK_ACCESS_TOKEN`. The opt-in is validated at startup; unset `SDKWORK_FORUM_SEARCH_MODE` (or set it to `in-process`) for the default in-process provider.

### Appbase Permission Mapping

Forum permission codes planned:

| Code | Description | Surface |
|------|-------------|---------|
| `forum.topics.create` | Create topics | app-api |
| `forum.topics.read` | Read topics | app-api, open-api |
| `forum.topics.update` | Update own topics | app-api |
| `forum.topics.delete` | Delete own topics | app-api |
| `forum.replies.create` | Create replies | app-api |
| `forum.replies.read` | Read replies | app-api, open-api |
| `forum.replies.update` | Update own replies | app-api |
| `forum.replies.delete` | Delete own replies | app-api |
| `forum.moderation.read` | Read moderation queue | backend-api |
| `forum.moderation.write` | Create decisions | backend-api |
| `forum.admin.nodes` | Manage taxonomy | backend-api |
| `forum.admin.reputation` | Manage reputation rules | backend-api |
| `forum.admin.badges` | Manage badges | backend-api |

Awaiting `sdkwork-iam-backend-sdk` dependency resolution for permission enforcement on backend routes. IAM session resolution is implemented in `sdkwork-api-forum-standalone-gateway` when enabled via env.
