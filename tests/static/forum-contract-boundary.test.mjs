import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();

const requiredFiles = [
  "AGENTS.md",
  "agent.app.config.json",
  "specs/component.spec.json",
  "specs/forum-database.schema.yaml",
  "apis/app-api/forum/openapi.yaml",
  "apis/backend-api/forum/openapi.yaml",
  "apis/open-api/forum/openapi.yaml",
  "sdks/sdkwork-forum-app-sdk/sdk-manifest.json",
  "sdks/sdkwork-forum-backend-sdk/sdk-manifest.json",
  "sdks/sdkwork-forum-sdk/sdk-manifest.json",
  "sdks/_route-manifests/app-api/sdkwork-routes-forum-app-api.route-manifest.json",
  "sdks/_route-manifests/backend-api/sdkwork-routes-forum-backend-api.route-manifest.json",
  "sdks/_route-manifests/open-api/sdkwork-routes-forum-open-api.route-manifest.json",
  "crates/sdkwork-communication-forum-service/src/lib.rs",
  "crates/sdkwork-routes-forum-app-api/src/lib.rs",
  "crates/sdkwork-routes-forum-backend-api/src/lib.rs",
  "crates/sdkwork-routes-forum-open-api/src/lib.rs"
];

const failures = [];

for (const file of requiredFiles) {
  if (!existsSync(join(root, file))) {
    failures.push(`missing required file: ${file}`);
  }
}

const contractFiles = [
  "specs/forum-database.schema.yaml",
  "apis/app-api/forum/openapi.yaml",
  "apis/backend-api/forum/openapi.yaml",
  "apis/open-api/forum/openapi.yaml",
  "sdks/_route-manifests/app-api/sdkwork-routes-forum-app-api.route-manifest.json",
  "sdks/_route-manifests/backend-api/sdkwork-routes-forum-backend-api.route-manifest.json",
  "sdks/_route-manifests/open-api/sdkwork-routes-forum-open-api.route-manifest.json"
];

for (const file of contractFiles) {
  if (!existsSync(join(root, file))) {
    continue;
  }
  const text = readFileSync(join(root, file), "utf8");
  if (/\bthreads?\b/i.test(text)) {
    failures.push(`forbidden forum term "thread" found in contract file: ${file}`);
  }
}

const openApiFile = join(root, "apis/open-api/forum/openapi.yaml");
if (existsSync(openApiFile)) {
  const openApi = readFileSync(openApiFile, "utf8");
  if (/Access-Token|AuthToken|X-Tenant|X-Organization/i.test(openApi)) {
    failures.push("open API must not declare SDKWork dual-token or business context headers");
  }
  if (/in:\s*header/i.test(openApi)) {
    failures.push("open API must not declare header parameters");
  }
  if (!/x-sdkwork-auth-mode:\s*anonymous/.test(openApi)) {
    failures.push("open API must mark public operations with x-sdkwork-auth-mode: anonymous");
  }
}

if (failures.length > 0) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log("forum contract boundary checks passed");
