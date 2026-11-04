/**
 * Tuiles « Valeurs des cartes » : une tuile par rang, de l'As au 10.
 */

// Importation des modules internes
import { formatTag } from "../../../format/number_format";

const RANK_LABELS = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10"];

function tileClass(tag: number): string {
  if (tag > 0) return "tile tile--positive";
  if (tag < 0) return "tile tile--negative";
  return "tile tile--neutral";
}

export default function CardValueTiles({ tags }: { tags: number[] }) {
  return (
    <div>
      <ul className="tiles" style={{ listStyle: "none", margin: 0, padding: 0 }}>
        {RANK_LABELS.map((label, i) => (
          <li key={label} className={tileClass(tags[i])} aria-label={`${label} : ${formatTag(tags[i])}`}>
            <span className="tile__card">{label}</span>
            <span className="tile__tag">{formatTag(tags[i])}</span>
          </li>
        ))}
      </ul>
      <p className="tiles-legend">
        <span>
          <i style={{ background: "var(--positive-bg)", border: "1px solid var(--positive-ink)" }} /> + : carte
          favorable au croupier qui sort, le sabot s&apos;améliore
        </span>
        <span>
          <i style={{ background: "var(--negative-bg)", border: "1px solid var(--negative-ink)" }} /> − : carte
          favorable au joueur qui sort
        </span>
        <span>
          <i style={{ background: "var(--neutral-bg)", border: "1px solid var(--neutral-ink)" }} /> 0 : neutre
        </span>
      </p>
      <p className="bar-note">Le 10 désigne toutes les cartes valant 10 (10, valet, dame, roi).</p>
    </div>
  );
}
