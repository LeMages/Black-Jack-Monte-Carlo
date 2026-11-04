// Importation des modules externes
import { describe, expect, it } from "vitest";

// Importation des modules internes
import {
  formatFractionAsPercent,
  formatInteger,
  formatN0,
  formatNumber,
  formatPercent,
  formatTag,
} from "./number_format";

/** Normalise les espaces insécables produits par Intl (fines ou non). */
function plain(s: string): string {
  return s.replace(/[\u202f\u00a0]/g, " ");
}

describe("formatage français", () => {
  it("utilise la virgule décimale", () => {
    expect(formatNumber(1.1409)).toBe("1,14");
    expect(formatNumber(2, 3)).toBe("2,000");
  });

  it("formate les pourcentages avec 2 décimales", () => {
    expect(plain(formatPercent(0.3146))).toBe("0,31 %");
    expect(plain(formatPercent(-0.358))).toBe("-0,36 %");
    expect(plain(formatPercent(0.5, 2, true))).toBe("+0,50 %");
    expect(plain(formatFractionAsPercent(0.0731))).toBe("7,31 %");
  });

  it("sépare les milliers de N₀", () => {
    expect(plain(formatInteger(226001.4))).toBe("226 001");
    expect(plain(formatN0(1234567))).toBe("1 234 567");
    expect(formatN0(null)).toBe("—");
  });

  it("affiche les tags avec signe et demi-points", () => {
    expect(formatTag(0.5)).toBe("+0,5");
    expect(formatTag(-2)).toBe("−2");
    expect(formatTag(0)).toBe("0");
    expect(formatTag(1.5)).toBe("+1,5");
  });
});
