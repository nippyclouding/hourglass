import { createHash } from "node:crypto";
import { readFile, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));

export async function renderCask({ repository, version, sha256 }) {
  if (!/^[A-Za-z0-9][A-Za-z0-9-]*\/[A-Za-z0-9_.-]+$/.test(repository ?? "")) {
    throw new Error("Use a GitHub owner/repository name, not a URL or placeholder.");
  }
  if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) throw new Error("Use a stable x.y.z version.");
  if (!/^[a-f0-9]{64}$/.test(sha256 ?? "")) throw new Error("A real SHA-256 checksum is required.");
  const template = await readFile(path.join(root, "packaging/hourglass.rb.template"), "utf8");
  return template.replaceAll("__REPOSITORY__", repository)
    .replaceAll("__VERSION__", version).replaceAll("__SHA256__", sha256);
}

async function main() {
  const options = {};
  for (let index = 2; index < process.argv.length; index += 2) {
    const key = process.argv[index];
    if (!["--repo", "--version", "--archive", "--output"].includes(key) || !process.argv[index + 1]) {
      throw new Error("Usage: npm run cask -- --repo OWNER/REPO --archive PATH [--version x.y.z] [--output PATH]");
    }
    options[key.slice(2)] = process.argv[index + 1];
  }
  const packageInfo = JSON.parse(await readFile(path.join(root, "package.json"), "utf8"));
  const version = options.version ?? packageInfo.version;
  if (!options.archive || path.basename(options.archive) !== `Hourglass_${version}_universal.zip`) {
    throw new Error(`Use the actual published Hourglass_${version}_universal.zip archive.`);
  }
  const sha256 = createHash("sha256").update(await readFile(options.archive)).digest("hex");
  const configuredRepo = packageInfo.repository?.url?.match(/^https:\/\/github\.com\/(.+)\.git$/)?.[1];
  const cask = await renderCask({ repository: options.repo ?? process.env.GITHUB_REPOSITORY ?? configuredRepo, version, sha256 });
  const output = path.resolve(options.output ?? path.join(root, "Casks/hourglass.rb"));
  await mkdir(path.dirname(output), { recursive: true });
  await writeFile(output, cask);
  console.log(`Generated ${output} with SHA-256 ${sha256}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
