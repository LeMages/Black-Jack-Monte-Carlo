/**
 * Captures d'écran du dashboard pour la documentation.
 *
 * Étapes : build du dashboard, lancement de `vite preview`, captures Playwright
 * (Chromium) des pages principales, arrêt du serveur.
 *
 * Usage (depuis la racine) : `npm --prefix dashboard run screenshots`.
 */

// Importation des modules standard
import { spawn, spawnSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const DASHBOARD = resolve(ROOT, "dashboard");
const OUT_DIR = resolve(ROOT, "docs", "screenshots");
const PORT = 4173;
const BASE_URL = `http://localhost:${PORT}`;
const CONFIG = "s15-i18f4";

// Playwright est une dépendance de développement du dashboard
const require = createRequire(resolve(DASHBOARD, "package.json"));
const { chromium } = require("playwright");

/** Pages à capturer : fichier, chemin, viewport, pleine page. */
const SHOTS = [
  { file: "accueil.png", path: `/?config=${CONFIG}`, width: 1440, height: 900, fullPage: true },
  { file: "systeme-hi-lo.png", path: `/systeme/hi-lo?config=${CONFIG}`, width: 1440, height: 900, fullPage: true },
  { file: "comparaison.png", path: `/comparaison?config=${CONFIG}`, width: 1440, height: 900, fullPage: true },
  { file: "mobile.png", path: `/systeme/hi-lo?config=${CONFIG}`, width: 390, height: 844, fullPage: false },
];

const NPM = process.platform === "win32" ? "npm.cmd" : "npm";

function build() {
  const res = spawnSync(NPM, ["run", "build"], { cwd: DASHBOARD, stdio: "inherit", shell: process.platform === "win32" });
  if (res.status !== 0) {
    throw new Error(`build du dashboard en échec (code ${res.status})`);
  }
}

/** Lance `vite preview` et attend que le serveur réponde. */
async function startPreview() {
  const server = spawn(NPM, ["run", "preview", "--", "--port", String(PORT), "--strictPort"], {
    cwd: DASHBOARD,
    stdio: "ignore",
    shell: process.platform === "win32",
  });
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      const res = await fetch(BASE_URL);
      if (res.ok) return server;
    } catch {
      // Serveur pas encore prêt : nouvel essai après une courte pause
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  server.kill();
  throw new Error(`vite preview ne répond pas sur ${BASE_URL} après 30 s`);
}

function stopServer(server) {
  if (process.platform === "win32") {
    // Sous Windows, le serveur tourne dans un shell : on tue l'arbre de processus
    spawnSync("taskkill", ["/pid", String(server.pid), "/T", "/F"], { stdio: "ignore" });
  } else {
    server.kill();
  }
}

async function capture() {
  mkdirSync(OUT_DIR, { recursive: true });
  const browser = await chromium.launch();
  try {
    for (const shot of SHOTS) {
      const page = await browser.newPage({ viewport: { width: shot.width, height: shot.height } });
      await page.goto(`${BASE_URL}${shot.path}`, { waitUntil: "networkidle" });
      await page.waitForSelector(".recharts-surface");
      await page.evaluate(() => document.fonts.ready);
      await page.screenshot({ path: resolve(OUT_DIR, shot.file), fullPage: shot.fullPage });
      console.log(`capture : docs/screenshots/${shot.file}`);
      await page.close();
    }
  } finally {
    await browser.close();
  }
}

build();
const server = await startPreview();
try {
  await capture();
} finally {
  stopServer(server);
}
