const test = require("node:test");
const assert = require("node:assert/strict");
const { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, chmodSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { spawnSync } = require("node:child_process");

const root = path.resolve(__dirname, "..");
const generator = import(pathToFileURL(path.join(root, "scripts/generate-cask.mjs")));

test("Cask uses a real version, release URL, checksum, and app artifact", async () => {
  const { renderCask } = await generator;
  const result = await renderCask({ repository: "example/hourglass", version: "0.1.0", sha256: "a".repeat(64) });
  assert.match(result, /app "Hourglass.app"/);
  assert.match(result, /example\/hourglass\/releases\/download\/v#\{version\}/);
  assert.match(result, /Hourglass_#\{version\}_universal.zip/);
  assert.match(result, /sha256 "a{64}"/);
  assert.doesNotMatch(result, /__REPOSITORY__|__VERSION__|__SHA256__|no_check/);
});

test("Cask rejects placeholders, Ruby interpolation, and unchecked archives", async () => {
  const { renderCask } = await generator;
  const options = { repository: "example/hourglass", version: "0.1.0", sha256: "a".repeat(64) };
  for (const repository of ["OWNER", "https://github.com/example/hourglass", 'example/#{system("bad")}', "../project"])
    await assert.rejects(renderCask({ ...options, repository }));
  await assert.rejects(renderCask({ ...options, sha256: "no_check" }));
  await assert.rejects(renderCask({ ...options, version: "latest" }));
});

function fixtureApp(directory, id, marker) {
  mkdirSync(path.join(directory, "Contents/MacOS"), { recursive: true });
  writeFileSync(path.join(directory, "Contents/Info.plist"), `<?xml version="1.0"?><plist version="1.0"><dict><key>CFBundleIdentifier</key><string>${id}</string></dict></plist>`);
  writeFileSync(path.join(directory, "Contents/MacOS/hourglass"), `#!/bin/sh\n# ${marker}\nexit 0\n`);
  chmodSync(path.join(directory, "Contents/MacOS/hourglass"), 0o755);
}

test("installer copies and upgrades the app without launching it", { skip: process.platform !== "darwin" }, () => {
  const temp = mkdtempSync(path.join(tmpdir(), "hourglass-install-test-"));
  try {
    const source = path.join(temp, "source/Hourglass.app");
    const appdir = path.join(temp, "Applications with spaces");
    fixtureApp(source, "io.hourglass.timer", "first");
    const run = () => spawnSync("/bin/bash", [path.join(root, "scripts/install-macos.sh"), "--app", source, "--appdir", appdir, "--no-start"], { encoding: "utf8" });
    let result = run();
    assert.equal(result.status, 0, result.stderr);
    const installed = path.join(appdir, "Hourglass.app/Contents/MacOS/hourglass");
    assert.match(readFileSync(installed, "utf8"), /first/);
    fixtureApp(source, "io.hourglass.timer", "updated");
    result = run();
    assert.equal(result.status, 0, result.stderr);
    assert.match(readFileSync(installed, "utf8"), /updated/);
  } finally { rmSync(temp, { recursive: true, force: true }); }
});

test("installer preserves another app occupying the destination", { skip: process.platform !== "darwin" }, () => {
  const temp = mkdtempSync(path.join(tmpdir(), "hourglass-install-test-"));
  try {
    const source = path.join(temp, "source/Hourglass.app");
    const appdir = path.join(temp, "Applications");
    fixtureApp(source, "io.hourglass.timer", "source");
    fixtureApp(path.join(appdir, "Hourglass.app"), "com.example.unrelated", "keep-me");
    const result = spawnSync("/bin/bash", [path.join(root, "scripts/install-macos.sh"), "--app", source, "--appdir", appdir, "--no-start"], { encoding: "utf8" });
    assert.notEqual(result.status, 0);
    assert.match(readFileSync(path.join(appdir, "Hourglass.app/Contents/MacOS/hourglass"), "utf8"), /keep-me/);
    assert.ok(existsSync(source));
  } finally { rmSync(temp, { recursive: true, force: true }); }
});
