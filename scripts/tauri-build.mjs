import { readFileSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import process from "node:process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const keyPath = path.join(__dirname, "tauri-signing.key");

const signed =
  process.argv.includes("--signed") ||
  Boolean(process.env.TAURI_SIGNING_PRIVATE_KEY) ||
  existsSync(keyPath);

const args = ["run", "tauri", "build"];
if (!signed) {
  args.push("--", "-c", '{"bundle":{"createUpdaterArtifacts":false}}');
  console.log(
    "Unsigned local build (no updater .sig / latest.json). Use npm run build:win:signed for release-parity.",
  );
} else if (!process.env.TAURI_SIGNING_PRIVATE_KEY && existsSync(keyPath)) {
  process.env.TAURI_SIGNING_PRIVATE_KEY = readFileSync(keyPath, "utf8");
}

const result = spawnSync("npm", args, {
  cwd: root,
  stdio: "inherit",
  shell: true,
  env: process.env,
});

process.exit(result.status ?? 1);
