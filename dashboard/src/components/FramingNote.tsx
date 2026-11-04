/**
 * Note de cadrage affichée en pied de page.
 */
export default function FramingNote() {
  return (
    <footer className="framing-note">
      <p>
        <strong>Note de cadrage.</strong> Ce site présente une étude de simulation, pas une méthode de jeu. En
        casino, le comptage est largement neutralisé : mélangeurs continus, pénétration réduite, contre-mesures
        contre les compteurs. Même dans les conditions simulées, l&apos;avantage reste inférieur à 1 %,
        noyé dans une variance énorme.
      </p>
    </footer>
  );
}
