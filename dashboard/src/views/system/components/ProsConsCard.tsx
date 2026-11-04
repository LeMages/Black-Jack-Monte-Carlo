/**
 * Carte « Forces et faiblesses ».
 */

// Importation des modules internes
import Card from "../../../components/Card";

interface Props {
  strengths: string[];
  weaknesses: string[];
}

export default function ProsConsCard({ strengths, weaknesses }: Props) {
  return (
    <Card title="Forces et faiblesses">
      <ul className="pros-cons">
        {strengths.map((s) => (
          <li key={s}>
            <span className="pros-cons__sign pros-cons__sign--plus" aria-label="Force">
              +
            </span>
            <span>{s}</span>
          </li>
        ))}
        {weaknesses.map((w) => (
          <li key={w}>
            <span className="pros-cons__sign pros-cons__sign--minus" aria-label="Faiblesse">
              −
            </span>
            <span>{w}</span>
          </li>
        ))}
      </ul>
    </Card>
  );
}
