import { realpathSync } from "node:fs";
import { spawn } from "node:child_process";

// If running inside an NTFS junction or symlink on Windows (e.g. C: -> H:),
// change directory to the physical target directory so Vite and SvelteKit
// compute intra-drive relative paths rather than invalid cross-drive paths (./H:/...).
try {
  const realCwd = realpathSync.native(process.cwd());
  if (process.cwd() !== realCwd) {
    process.chdir(realCwd);
  }
} catch (_) {}

const args = process.argv.slice(2);
const bin = process.platform === "win32" ? "npx.cmd" : "npx";
const child = spawn(bin, args, { stdio: "inherit", shell: true });

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 0);
  }
});
