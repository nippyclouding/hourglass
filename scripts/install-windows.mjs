import { readFile, readdir } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import path from "node:path";

if (process.platform !== "win32") {
  console.error("Run this installer on Windows; macOS uses npm run install:macos.");
  process.exit(1);
}
const root = fileURLToPath(new URL("../", import.meta.url));
const { version } = JSON.parse(await readFile(path.join(root, "package.json"), "utf8"));
const directory = path.join(root, "src-tauri/target/release/bundle/nsis");
const installers = (await readdir(directory))
  .filter(name => name.startsWith(`Hourglass_${version}_`) && name.endsWith("-setup.exe"));
if (installers.length !== 1) {
  throw new Error("Expected exactly one installer for this app version. Build the NSIS bundle first.");
}
const result = spawnSync(path.join(directory, installers[0]), [], { stdio: "inherit" });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
