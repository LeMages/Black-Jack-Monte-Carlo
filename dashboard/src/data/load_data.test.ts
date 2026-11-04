// Importation des modules standard
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

// Importation des modules externes
import { describe, expect, it } from "vitest";

// Importation des modules internes
import { buildBundle, DATA_FILES, DataContractError, loadData, SYSTEM_IDS } from "./load_data";

const DATA_DIR = resolve(__dirname, "../../public/data");

function readRaw(): Record<string, unknown> {
  return Object.fromEntries(
    DATA_FILES.map((name) => [name, JSON.parse(readFileSync(resolve(DATA_DIR, `${name}.json`), "utf8"))]),
  );
}

describe("données publiées", () => {
  it("respectent le contrat et couvrent chaque (système × config)", () => {
    const data = buildBundle(readRaw());
    expect(data.systems.map((s) => s.id)).toEqual(SYSTEM_IDS);
    for (const config of data.configs) {
      for (const system of SYSTEM_IDS) {
        const rows = data.metrics.filter((m) => m.system === system && m.config_id === config.id);
        expect(rows).toHaveLength(1);
        expect(data.equityCurves.some((r) => r.system === system && r.config_id === config.id)).toBe(true);
        expect(data.tcDistribution.some((r) => r.system === system && r.config_id === config.id)).toBe(true);
      }
    }
    expect(data.baseline.system).toBe("basic");
    expect(data.meta.total_hands_simulated).toBeGreaterThan(0);
  });

  it("chaque système a 10 tags", () => {
    const data = buildBundle(readRaw());
    for (const s of data.systems) {
      expect(s.tags).toHaveLength(10);
    }
  });
});

describe("validation du contrat", () => {
  it("rejette une colonne manquante", () => {
    const raw = readRaw();
    const metrics = (raw.metrics as Record<string, unknown>[]).map((r) => ({ ...r }));
    delete metrics[0].advantage_pct;
    expect(() => buildBundle({ ...raw, metrics })).toThrow(DataContractError);
  });

  it("rejette un système inconnu", () => {
    const raw = readRaw();
    const metrics = (raw.metrics as Record<string, unknown>[]).map((r) => ({ ...r }));
    metrics[0].system = "inconnu";
    expect(() => buildBundle({ ...raw, metrics })).toThrow(/system/);
  });

  it("accepte n0 null", () => {
    const raw = readRaw();
    const metrics = (raw.metrics as Record<string, unknown>[]).map((r) => ({ ...r, n0: null }));
    expect(() => buildBundle({ ...raw, metrics })).not.toThrow();
  });

  it("signale un fichier absent au chargement", async () => {
    const fetcher = (async () => new Response("", { status: 404 })) as typeof fetch;
    await expect(loadData("/data", fetcher)).rejects.toThrow(/HTTP 404/);
  });

  it("charge via fetch", async () => {
    const raw = readRaw();
    const fetcher = (async (url: string) => {
      const name = url.replace("/data/", "").replace(".json", "");
      return new Response(JSON.stringify(raw[name]), { status: 200 });
    }) as typeof fetch;
    const data = await loadData("/data", fetcher);
    expect(data.configs.length).toBeGreaterThan(0);
  });
});
