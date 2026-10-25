import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const failures = [];

function fail(message) {
  failures.push(message);
}

function read(file) {
  return readFileSync(join(root, file), "utf8");
}

function readJson(file) {
  return JSON.parse(read(file));
}

const requiredFiles = [
  "AGENTS.md",
  "agent.app.config.json",
  "specs/component.spec.json",
  "specs/forum-database.schema.yaml",
  "docs/forum-database-design.md",
  "docs/forum-api-design.md",
  "apis/app-api/forum/openapi.yaml",
  "apis/backend-api/forum/openapi.yaml",
  "apis/open-api/forum/openapi.yaml",
  "sdks/sdkwork-forum-app-sdk/openapi/sdkwork-forum-app-api.openapi.yaml",
  "sdks/sdkwork-forum-backend-sdk/openapi/sdkwork-forum-backend-api.openapi.yaml",
  "sdks/sdkwork-forum-sdk/openapi/sdkwork-forum-open-api.openapi.yaml",
  "sdks/sdkwork-forum-app-sdk/sdk-manifest.json",
  "sdks/sdkwork-forum-backend-sdk/sdk-manifest.json",
  "sdks/sdkwork-forum-sdk/sdk-manifest.json",
  "sdks/_route-manifests/app-api/sdkwork-routes-forum-app-api.route-manifest.json",
  "sdks/_route-manifests/backend-api/sdkwork-routes-forum-backend-api.route-manifest.json",
  "sdks/_route-manifests/open-api/sdkwork-routes-forum-open-api.route-manifest.json",
  "crates/sdkwork-communication-forum-service/src/lib.rs",
  "crates/sdkwork-communication-forum-repository-sqlx/src/lib.rs",
  "crates/sdkwork-routes-forum-app-api/src/lib.rs",
  "crates/sdkwork-routes-forum-backend-api/src/lib.rs",
  "crates/sdkwork-routes-forum-open-api/src/lib.rs"
];

for (const file of requiredFiles) {
  if (!existsSync(join(root, file))) {
    fail(`missing required file: ${file}`);
  }
}

const strictContractFiles = [
  "specs/forum-database.schema.yaml",
  "apis/app-api/forum/openapi.yaml",
  "apis/backend-api/forum/openapi.yaml",
  "apis/open-api/forum/openapi.yaml",
  "sdks/sdkwork-forum-app-sdk/openapi/sdkwork-forum-app-api.openapi.yaml",
  "sdks/sdkwork-forum-backend-sdk/openapi/sdkwork-forum-backend-api.openapi.yaml",
  "sdks/sdkwork-forum-sdk/openapi/sdkwork-forum-open-api.openapi.yaml",
  "sdks/_route-manifests/app-api/sdkwork-routes-forum-app-api.route-manifest.json",
  "sdks/_route-manifests/backend-api/sdkwork-routes-forum-backend-api.route-manifest.json",
  "sdks/_route-manifests/open-api/sdkwork-routes-forum-open-api.route-manifest.json"
];

for (const file of strictContractFiles) {
  if (!existsSync(join(root, file))) continue;
  const text = read(file);
  if (/\bthreads?\b/i.test(text)) {
    fail(`forbidden forum resource term found in strict contract file: ${file}`);
  }
}

const db = existsSync(join(root, "specs/forum-database.schema.yaml"))
  ? read("specs/forum-database.schema.yaml")
  : "";
if (db) {
  const tableMatches = [...db.matchAll(/^\s+- name: (forum_[a-z0-9_]+)/gm)].map((match) => match[1]);
  const requiredTables = [
    "forum_space",
    "forum_node",
    "forum_board_profile",
    "forum_tag",
    "forum_topic_tag",
    "forum_topic_prefix",
    "forum_node_acl",
    "forum_topic",
    "forum_topic_revision",
    "forum_topic_reply",
    "forum_reply_revision",
    "forum_attachment",
    "forum_question_profile",
    "forum_poll",
    "forum_poll_option",
    "forum_poll_vote",
    "forum_reaction",
    "forum_vote",
    "forum_bookmark",
    "forum_subscription",
    "forum_read_state",
    "forum_notification_preference",
    "forum_member_profile",
    "forum_trust_level",
    "forum_privilege_grant",
    "forum_badge",
    "forum_user_badge",
    "forum_reputation_ledger",
    "forum_reputation_rule",
    "forum_report",
    "forum_moderation_queue_item",
    "forum_moderation_case",
    "forum_moderation_decision",
    "forum_moderation_policy",
    "forum_sanction",
    "forum_appeal",
    "forum_feed_item",
    "forum_public_topic_projection",
    "forum_topic_stats",
    "forum_board_stats",
    "forum_member_stats",
    "forum_search_document",
    "forum_outbox_event",
    "forum_inbox_event",
    "forum_idempotency_record"
  ];
  for (const table of requiredTables) {
    if (!tableMatches.includes(table)) {
      fail(`missing database table contract: ${table}`);
    }
  }
  for (const table of tableMatches) {
    if (!table.startsWith("forum_")) {
      fail(`database table does not use forum_ prefix: ${table}`);
    }
  }
}

const openApiFile = "apis/open-api/forum/openapi.yaml";
if (existsSync(join(root, openApiFile))) {
  const openApi = read(openApiFile);
  if (/Access-Token|AuthToken|X-Tenant|X-Organization/i.test(openApi)) {
    fail("open API declares SDKWork dual-token or business context headers");
  }
  if (/in:\s*header/i.test(openApi)) {
    fail("open API declares header parameters");
  }
  if (!/security:\s*\[\]/.test(openApi)) {
    fail("open API must explicitly declare security: [] on public operations");
  }
  if (!/x-sdkwork-auth-mode:\s*anonymous/.test(openApi)) {
    fail("open API must mark public operations with x-sdkwork-auth-mode: anonymous");
  }
}

const assemblies = [
  ["sdks/sdkwork-forum-app-sdk/sdk-manifest.json", "sdkwork-forum", "sdkwork-forum-app-api", "sdkwork-forum-app-sdk", "/app/v3/api"],
  ["sdks/sdkwork-forum-backend-sdk/sdk-manifest.json", "sdkwork-forum", "sdkwork-forum-backend-api", "sdkwork-forum-backend-sdk", "/backend/v3/api"],
  ["sdks/sdkwork-forum-sdk/sdk-manifest.json", "sdkwork-forum", "sdkwork-forum-open-api", "sdkwork-forum-sdk", "/forum/v3/api"]
];

for (const [file, owner, authority, family, prefix] of assemblies) {
  if (!existsSync(join(root, file))) continue;
  const json = readJson(file);
  if (json.sdkOwner !== owner) fail(`${file} sdkOwner mismatch`);
  if (json.apiAuthority !== authority) fail(`${file} apiAuthority mismatch`);
  if (json.sdkFamily !== family) fail(`${file} sdkFamily mismatch`);
  if (json.discoverySurface?.apiPrefix !== prefix) fail(`${file} apiPrefix mismatch`);
  if (!Array.isArray(json.sdkDependencies)) fail(`${file} sdkDependencies must be explicit array`);
}

const manifestSpecs = [
  ["sdks/_route-manifests/app-api/sdkwork-routes-forum-app-api.route-manifest.json", "app-api", "/app/v3/api"],
  ["sdks/_route-manifests/backend-api/sdkwork-routes-forum-backend-api.route-manifest.json", "backend-api", "/backend/v3/api"],
  ["sdks/_route-manifests/open-api/sdkwork-routes-forum-open-api.route-manifest.json", "open-api", "/forum/v3/api"]
];

for (const [file, surface, prefix] of manifestSpecs) {
  if (!existsSync(join(root, file))) continue;
  const json = readJson(file);
  if (json.kind !== "sdkwork.route.manifest") fail(`${file} kind mismatch`);
  if (json.surface !== surface) fail(`${file} surface mismatch`);
  if (json.domain !== "communication") fail(`${file} domain mismatch`);
  if (json.capability !== "forum") fail(`${file} capability mismatch`);
  if (json.prefix !== prefix) fail(`${file} prefix mismatch`);
  if (!Array.isArray(json.routes) || json.routes.length === 0) fail(`${file} routes must be non-empty`);
  const seen = new Set();
  for (const route of json.routes ?? []) {
    if (!route.path?.startsWith(prefix)) fail(`${file} route path outside prefix: ${route.path}`);
    const key = `${route.method} ${route.path}`;
    if (seen.has(key)) fail(`${file} duplicate route: ${key}`);
    seen.add(key);
    if (!route.operationId || !/^[a-z][A-Za-z0-9]*(\.[a-z][A-Za-z0-9]*)+$/.test(route.operationId)) {
      fail(`${file} invalid operationId: ${route.operationId}`);
    }
    if (surface === "open-api" && route.auth?.mode !== "public") {
      fail(`${file} open route must use public auth mode: ${route.operationId}`);
    }
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("forum contract validation passed");
