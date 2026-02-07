import { spawn } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "../..");

// Path to the debug binary produced by `cargo tauri build --debug --no-bundle`
const appBinary = resolve(
  repoRoot,
  "src-tauri/target/debug/agents-manager",
);

let tauriDriver;

export const config = {
  specs: ["./specs/**/*.js"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": {
        application: appBinary,
      },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: {
    ui: "bdd",
    timeout: 60_000,
  },

  // Spawn tauri-driver before each session
  beforeSession() {
    tauriDriver = spawn("tauri-driver", [], {
      stdio: [null, process.stdout, process.stderr],
    });

    // Give the driver a moment to start
    return new Promise((r) => setTimeout(r, 1500));
  },

  // Kill tauri-driver after each session
  afterSession() {
    tauriDriver?.kill();
  },
};
