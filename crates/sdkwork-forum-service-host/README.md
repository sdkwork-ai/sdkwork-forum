# SDKWork Forum Service Host

Dependency composition for forum service runtime.

## Implementation Status

- `ForumServiceHost`: Wraps `ForumService<SqlxForumRepository>` with `service()` accessor and `build_request_context()` factory. `new()` and `from_database_pool()` return `Result` and fail at startup when an explicitly selected split deployment is misconfigured.
- `build_forum_service()`: Constructs `ForumService<SqlxForumRepository>` with placeholder repository.

## Sibling Module Ports (MODULE_SPEC.md §4.2)

`src/ports.rs` composes the outbound ports on the host `PgPool`:

| Port | Default | Opt-in |
| --- | --- | --- |
| `ForumDrivePort` | `InProcessForumDrivePort`: sdkwork-drive `DriveUploaderStore::find_active_node`/`find_active_space` + `SqlDrivePermissionService::resolve_effective_node_access` (`reader`) + `build_download_token` | none — Drive has no HTTP transport |
| `ForumSearchPort` | `InProcessForumSearchPort`: sdkwork-search provider SPI (`PostgresqlSearchProvider`) on the shared pool | `SDKWORK_FORUM_SEARCH_MODE=http` + `SDKWORK_FORUM_SEARCH_URL` + `SDKWORK_ACCESS_TOKEN`, validated at startup |

Both in-process ports receive the `ForumRequestContext`, so the tenant and the caller's subject come from the request, never from an environment default. The retired `SDKWORK_FORUM_DRIVE_URL` variable was dead configuration (it selected a logging no-op) and is deleted.
