import net from "node:net";
import { spawn } from "node:child_process";

const host = "127.0.0.1";
const port = 1420;

function isPortOpen() {
  return new Promise((resolve) => {
    const socket = net.createConnection({ host, port });
    const finish = (open) => {
      socket.destroy();
      resolve(open);
    };
    socket.once("connect", () => finish(true));
    socket.once("error", () => finish(false));
    socket.setTimeout(1000, () => finish(false));
  });
}

if (await isPortOpen()) {
  console.log(`Vite is already running on ${host}:${port}. Reusing it.`);
  process.exit(0);
}

const isWindows = process.platform === "win32";
const command = isWindows ? (process.env.ComSpec ?? "cmd.exe") : "corepack";
const args = isWindows
  ? ["/d", "/s", "/c", `corepack pnpm dev --host ${host}`]
  : ["pnpm", "dev", "--host", host];
const child = spawn(command, args, {
  cwd: process.cwd(),
  stdio: "inherit",
  shell: false,
});

child.on("exit", (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  process.exit(code ?? 1);
});

child.on("error", (error) => {
  console.error(`Failed to start Vite: ${error.message}`);
  process.exit(1);
});
