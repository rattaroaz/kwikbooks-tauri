import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const REPO_ROOT = resolve(import.meta.dirname, "../..");

/** IPC command names registered in `src-tauri/src/lib.rs`. */
export function registeredRustCommands(): string[] {
  const lib = readFileSync(resolve(REPO_ROOT, "src-tauri/src/lib.rs"), "utf8");
  const block = lib.match(/generate_handler!\[([\s\S]*?)\]/)?.[1] ?? "";
  const names: string[] = [];
  for (const line of block.split("\n")) {
    const trimmed = line
      .replace(/\/\/.*$/, "")
      .trim()
      .replace(/,$/, "");
    if (!trimmed) {
      continue;
    }
    const name = trimmed.includes("::") ? trimmed.split("::").pop()! : trimmed;
    names.push(name);
  }
  return [...new Set(names)].sort();
}

/** IPC command names invoked from the TypeScript API layer. */
export function frontendInvokeCommands(): string[] {
  const files = ["src/api/tauri.ts", "src/api/db.ts"];
  const names = new Set<string>();
  for (const rel of files) {
    const src = readFileSync(resolve(REPO_ROOT, rel), "utf8");
    for (const match of src.matchAll(/invoke(?:<[^>]*>)?\("([^"]+)"/g)) {
      names.add(match[1]!);
    }
  }
  return [...names].sort();
}

/** Command names handled by the Playwright E2E Tauri mock (excludes plugin bridges). */
export function e2eMockCommands(): string[] {
  const src = readFileSync(
    resolve(REPO_ROOT, "tests/e2e/tauriMock.ts"),
    "utf8",
  );
  const names = new Set<string>();
  for (const match of src.matchAll(/case "([^"]+)":/g)) {
    names.add(match[1]!);
  }
  return [...names].sort();
}

export function missingFromE2eMock(): string[] {
  const appCommands = registeredRustCommands();
  const mocked = new Set(e2eMockCommands());
  return appCommands.filter((cmd) => !mocked.has(cmd));
}

export function frontendNotInRust(): string[] {
  const rust = new Set(registeredRustCommands());
  return frontendInvokeCommands().filter((cmd) => !rust.has(cmd));
}
