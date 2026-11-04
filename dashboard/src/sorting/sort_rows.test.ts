// Importation des modules externes
import { describe, expect, it } from "vitest";

// Importation des modules internes
import { ariaSort, nextSortState, sortIndicator, sortRows, type SortValue } from "./sort_rows";

interface Row {
  name: string;
  adv: number;
  n0: number | null;
}

const rows: Row[] = [
  { name: "Zen", adv: 0.3, n0: 180000 },
  { name: "Hi-Lo", adv: 0.28, n0: null },
  { name: "KO", adv: 0.5, n0: 99000 },
];

const get = (row: Row, key: keyof Row): SortValue => row[key];

describe("tri du tableau", () => {
  it("trie les nombres dans les deux sens", () => {
    expect(sortRows(rows, get, { key: "adv", direction: "desc" }).map((r) => r.name)).toEqual(["KO", "Zen", "Hi-Lo"]);
    expect(sortRows(rows, get, { key: "adv", direction: "asc" }).map((r) => r.name)).toEqual(["Hi-Lo", "Zen", "KO"]);
  });

  it("place toujours null en dernier", () => {
    expect(sortRows(rows, get, { key: "n0", direction: "asc" }).map((r) => r.name)).toEqual(["KO", "Zen", "Hi-Lo"]);
    expect(sortRows(rows, get, { key: "n0", direction: "desc" }).map((r) => r.name)).toEqual(["Zen", "KO", "Hi-Lo"]);
  });

  it("trie le texte en français et ne modifie pas l'entrée", () => {
    const copy = [...rows];
    expect(sortRows(rows, get, { key: "name", direction: "asc" }).map((r) => r.name)).toEqual(["Hi-Lo", "KO", "Zen"]);
    expect(rows).toEqual(copy);
  });

  it("alterne le sens au clic et expose aria-sort", () => {
    const first = nextSortState<keyof Row>(null, "adv", true);
    expect(first).toEqual({ key: "adv", direction: "desc" });
    const second = nextSortState(first, "adv", true);
    expect(second.direction).toBe("asc");
    expect(nextSortState(second, "name", false)).toEqual({ key: "name", direction: "asc" });
    expect(ariaSort(second, "adv")).toBe("ascending");
    expect(ariaSort(second, "n0")).toBe("none");
    expect(sortIndicator(first, "adv")).toBe("↓");
    expect(sortIndicator(first, "n0")).toBe("↕");
  });
});
