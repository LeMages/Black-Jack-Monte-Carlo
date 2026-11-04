// Importation des modules externes
import { describe, expect, it } from "vitest";

// Importation des modules internes
import { buildCountingExample, cardRank, EXAMPLE_SEQUENCE } from "./counting_example";
import type { SystemDefinition } from "../types/data";

function system(overrides: Partial<SystemDefinition>): SystemDefinition {
  return {
    id: "hi-lo",
    name: "Hi-Lo",
    tags: [-1, 1, 1, 1, 1, 1, 0, 0, 0, -1],
    balanced: true,
    level: 1,
    ace_side_count: false,
    betting_correlation: 0.97,
    playing_efficiency: 0.51,
    insurance_correlation: 0.76,
    initial_running_count: 0,
    pivot: null,
    index_scale: 1,
    ace_factor: null,
    ...overrides,
  };
}

describe("exemples de comptage", () => {
  it("reproduit l'exemple Hi-Lo de l'annexe (+2, TC +0,5)", () => {
    const ex = buildCountingExample(system({}));
    expect(ex.rows.map((r) => r.running)).toEqual([1, 2, 1, 1, 0, -1, -1, 0, 1, 2]);
    expect(ex.finalRunning).toBe(2);
    expect(ex.trueCount).toBe(0.5);
    expect(ex.pivot).toBeNull();
    expect(ex.aceCorrection).toBeNull();
  });

  it("part de l'IRC et compare au pivot pour KO", () => {
    const ko = system({
      id: "ko",
      tags: [-1, 1, 1, 1, 1, 1, 1, 0, 0, -1],
      balanced: false,
      initial_running_count: -20,
      pivot: 4,
    });
    const ex = buildCountingExample(ko);
    expect(ex.initialRunning).toBe(-20);
    expect(ex.finalRunning).toBe(-17);
    expect(ex.trueCount).toBeNull();
    expect(ex.pivot).toEqual({ value: 4, reached: false });
  });

  it("calcule la correction du side-count des As", () => {
    const hopt = system({
      id: "hi-opt-ii",
      tags: [0, 1, 1, 2, 2, 1, 1, 0, 0, -2],
      ace_side_count: true,
      ace_factor: 2,
    });
    const ex = buildCountingExample(hopt, EXAMPLE_SEQUENCE, { nDecks: 6, decksRemaining: 4, acesSeenBefore: 4 });
    expect(ex.finalRunning).toBe(5);
    expect(ex.trueCount).toBe(1.25);
    // 5 As sortis → 19 restants pour 16 attendus : +3, correction 2 × 3 / 4 = +1,5
    expect(ex.aceCorrection).toMatchObject({ acesSeen: 5, acesRemaining: 19, acesRich: 3, correction: 1.5 });
    expect(ex.aceCorrection?.bettingCount).toBe(2.75);
  });

  it("gère les demi-points du Wong Halves", () => {
    const wong = system({ id: "wong-halves", tags: [-1, 0.5, 1, 1, 1.5, 1, 0.5, 0, -0.5, -1] });
    expect(buildCountingExample(wong).finalRunning).toBe(3);
  });

  it("convertit les cartes affichées", () => {
    expect(cardRank("A")).toBe(1);
    expect(cardRank("K")).toBe(10);
    expect(cardRank("7")).toBe(7);
    expect(() => cardRank("Z")).toThrow();
  });
});
