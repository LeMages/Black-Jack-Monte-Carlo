/**
 * Section « Comment ça marche — exemple » : tableau Carte / Tag / Running et
 * encadré du true count (ou du pivot pour KO, de la correction des As pour les
 * systèmes à side-count). Tout est calculé par `buildCountingExample`.
 */

// Importation des modules internes
import Card from "../../../components/Card";
import { buildCountingExample, DEFAULT_ASSUMPTIONS } from "../../../counting/counting_example";
import { formatNumber, formatTag } from "../../../format/number_format";
import type { SystemDefinition } from "../../../types/data";

function signed(value: number): string {
  return formatTag(value);
}

function decimal(value: number): string {
  return formatNumber(value, 2).replace("-", "−");
}

export default function CountingExampleCard({ system }: { system: SystemDefinition }) {
  const example = buildCountingExample(system);
  const decks = formatNumber(example.decksRemaining, 0);
  return (
    <Card title="Comment ça marche — exemple">
      <div className="example-layout">
        <div className="table-scroll">
          <table className="data-table">
            <caption className="muted" style={{ textAlign: "left", marginBottom: 8 }}>
              Running count de départ : {signed(example.initialRunning)}
              {system.balanced ? "" : " (IRC pour 6 jeux)"}
            </caption>
            <thead>
              <tr>
                <th scope="col" style={{ padding: "10px 12px" }}>
                  Carte
                </th>
                <th scope="col" style={{ padding: "10px 12px" }}>
                  Tag
                </th>
                <th scope="col" style={{ padding: "10px 12px" }}>
                  Running
                </th>
              </tr>
            </thead>
            <tbody>
              {example.rows.map((row, i) => (
                <tr key={i}>
                  <td>{row.card}</td>
                  <td>{signed(row.tag)}</td>
                  <td>{signed(row.running)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div className="example-box">
          {example.trueCount !== null && (
            <div className="callout">
              <p className="formula">
                Running count {signed(example.finalRunning)} ÷ {decks} jeux restants = True count{" "}
                {signed(Number(example.trueCount.toFixed(2)))}
              </p>
            </div>
          )}
          {example.pivot && (
            <div className="callout">
              <p className="formula">
                Running count {signed(example.finalRunning)} {example.pivot.reached ? "≥" : "<"} pivot{" "}
                {signed(example.pivot.value)}
              </p>
              <p>
                {example.pivot.reached
                  ? "Le pivot est atteint : la mise monte."
                  : "Le pivot n'est pas atteint : mise minimale. Aucune division par les jeux restants."}
              </p>
            </div>
          )}
          {example.aceCorrection && (
            <div className="callout">
              <p>
                <strong>Side-count des As.</strong> Si {example.aceCorrection.acesSeen} As sont déjà sortis
                (dont celui de la séquence), il en reste {example.aceCorrection.acesRemaining} pour{" "}
                {example.aceCorrection.acesExpected} attendus (4 × {decks} jeux) :{" "}
                {signed(example.aceCorrection.acesRich)} As.
              </p>
              <p className="formula">
                Correction = {formatNumber(example.aceCorrection.aceFactor, 0)} ×{" "}
                {signed(example.aceCorrection.acesRich)} ÷ {decks} = {signed(example.aceCorrection.correction)}
              </p>
              <p className="formula">
                Compte de mise = {decimal(example.trueCount ?? 0)} {example.aceCorrection.correction < 0 ? "−" : "+"}{" "}
                {decimal(Math.abs(example.aceCorrection.correction))} ={" "}
                {decimal(example.aceCorrection.bettingCount)}
              </p>
              <p className="muted">Le compte de jeu (déviations) reste à {decimal(example.trueCount ?? 0)}.</p>
            </div>
          )}
          <p className="muted">
            Hypothèse : sabot de {DEFAULT_ASSUMPTIONS.nDecks} jeux, {decks} jeux restant à distribuer au moment
            du calcul. En simulation, le nombre de jeux restants est exact ; un joueur l&apos;estime au demi-jeu
            près.
          </p>
        </div>
      </div>
    </Card>
  );
}
