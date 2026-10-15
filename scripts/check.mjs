import { readFile, stat } from "node:fs/promises";
import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import path from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
const config = JSON.parse(await readFile(path.join(root, "src-tauri/tauri.conf.json"), "utf8"));
const capability = JSON.parse(await readFile(path.join(root, "src-tauri/capabilities/settings.json"), "utf8"));
const completionCapability = JSON.parse(await readFile(path.join(root, "src-tauri/capabilities/completion.json"), "utf8"));
assert.deepEqual(config.app.windows, [], "Startup must not create a webview");
assert.equal(config.build.frontendDist, "../ui");
assert.deepEqual(capability.windows, ["settings"]);
assert.deepEqual(completionCapability.windows, ["completion"]);
assert.ok(!config.app.security.csp.includes("unsafe-eval"));
assert.ok(!config.app.security.csp.includes("unsafe-inline"));
for (const icon of config.bundle.icon) await stat(path.join(root, "src-tauri", icon));
for (const icon of ["tray.rgba", "tray-windows.rgba"]) {
  const bytes = await readFile(path.join(root, "src-tauri/icons", icon));
  assert.equal(bytes.length, 32 * 32 * 4);
  assert.ok(bytes.some((value, index) => index % 4 === 3 && value > 0));
}
for (const icon of ["tray-animation.rgba", "tray-animation-windows.rgba"]) {
  const bytes = await readFile(path.join(root, "src-tauri/icons", icon));
  const stride = 32 * 32 * 4;
  assert.equal(bytes.length, stride * 8 * 4);
  assert.ok(!bytes.subarray(0, stride).equals(bytes.subarray(stride, stride * 2)), "Grains must move");
  assert.ok(!bytes.subarray(0, stride).equals(bytes.subarray(stride * 28, stride * 29)), "Sand must drain");
}
for (const file of ["ui/settings.js", "ui/completion.js", "tests/settings.test.cjs", "tests/completion.test.cjs", "tests/distribution.test.cjs", "scripts/generate-cask.mjs", "scripts/install-windows.mjs"]) {
  const result = spawnSync(process.execPath, ["--check", path.join(root, file)], { stdio: "inherit" });
  assert.equal(result.status, 0);
}
console.log("Frontend syntax, tray assets, and local app configuration OK.");
