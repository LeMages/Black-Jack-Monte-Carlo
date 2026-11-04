/**
 * Contenu pédagogique des pages-rapport (texte uniquement).
 *
 * Les valeurs numériques (tags, BC/PE/IC, IRC, pivot) ne sont pas dupliquées
 * ici : elles proviennent de `systems.json`, exporté par le moteur. Les
 * résultats chiffrés proviennent de `metrics.json` et des autres agrégats.
 */

// Importation des modules internes
import type { SystemId } from "../types/data";

/** Texte d'une page-rapport. */
export interface SystemContent {
  /** Slug d'URL (identique à l'identifiant moteur). */
  slug: SystemId;
  /** Auteur(s) et année de publication. */
  author: string;
  year: number;
  /** Accroche affichée sur la carte de l'accueil. */
  tagline: string;
  /** Paragraphes de « En quoi ça consiste ». */
  paragraphs: string[];
  /** Points forts. */
  strengths: string[];
  /** Points faibles. */
  weaknesses: string[];
}

export const SYSTEMS_CONTENT: Record<SystemId, SystemContent> = {
  "hi-lo": {
    slug: "hi-lo",
    author: "Harvey Dubner",
    year: 1963,
    tagline: "La référence, le meilleur compromis simplicité / performance.",
    paragraphs: [
      "Le Hi-Lo est le système de référence du comptage de cartes, et de loin le plus documenté. Présenté par Harvey Dubner en 1963 puis popularisé par Julian Braun et Stanford Wong, il sert d'étalon à presque toutes les études : les index de déviations les plus connus (Illustrious 18, Fab 4) sont publiés pour lui.",
      "Son principe tient en une phrase : les cartes basses (2 à 6), favorables au croupier, valent +1 quand elles sortent ; les cartes hautes (10 et As), favorables au joueur, valent −1 ; les neutres (7, 8, 9) valent 0. Un running count positif signifie qu'il reste proportionnellement plus de cartes hautes dans le sabot.",
      "Le running count est ensuite divisé par le nombre de jeux restants pour obtenir le true count, qui pilote à la fois la mise et les déviations. C'est le système conseillé à qui débute : un seul niveau, aucun side-count, et une corrélation de pari parmi les meilleures.",
      "Dans ce projet, Hi-Lo est aussi le point de comparaison : la rampe de mise et les index des autres systèmes équilibrés sont ramenés à son échelle pour que la comparaison porte sur la qualité de l'information, pas sur l'agressivité de la mise.",
    ],
    strengths: [
      "Excellente corrélation de pari (BC .97) : il sait quand miser gros.",
      "Simple : un seul niveau, pas de fraction, pas de side-count.",
      "Index de déviations publiés et vérifiés (I18, Fab 4).",
    ],
    weaknesses: [
      "Efficacité de jeu modeste (PE .51) : déviations moins fines.",
      "Demande la conversion running count → true count à chaque décision.",
      "Ignore le 7, le 8 et le 9, qui portent pourtant un peu d'information.",
    ],
  },
  ko: {
    slug: "ko",
    author: "Olaf Vancura et Ken Fuchs",
    year: 1998,
    tagline: "Non équilibré : on mise au running count, sans division.",
    paragraphs: [
      "Le KO (Knock-Out), publié par Olaf Vancura et Ken Fuchs dans Knock-Out Blackjack (1998), part d'un constat : la division du running count par le nombre de jeux restants est l'étape la plus coûteuse et la plus source d'erreurs pour un joueur humain. Le KO la supprime.",
      "Pour cela, le 7 compte +1 en plus des cartes de 2 à 6. Le compte n'est plus équilibré : un jeu complet totalise +4. En partant d'un running count initial (IRC) négatif, −20 pour 6 jeux, le running count dérive naturellement vers le haut à mesure que le sabot avance, ce qui compense l'absence de division.",
      "La règle de mise devient une simple comparaison : dès que le running count atteint le pivot (+4), le sabot est favorable et la mise monte. Les déviations utilisent des index propres au KO ; dans ce projet, elles reposent sur une conversion exacte du running count KO en compte équilibré.",
      "Le KO vise les joueurs qui veulent l'essentiel du gain d'un système équilibré niveau 1, avec moins de calcul mental et donc moins d'erreurs à la table.",
    ],
    strengths: [
      "Aucune conversion en true count : rapide, peu d'erreurs.",
      "Corrélation de pari très élevée (BC .98).",
      "Le pivot donne un repère de mise unique, indépendant de la profondeur.",
    ],
    weaknesses: [
      "Précision de mise variable selon la profondeur du sabot.",
      "Une même hausse du running count pèse plus en début qu'en fin de sabot.",
      "Index de déviations moins standardisés que ceux du Hi-Lo.",
    ],
  },
  "hi-opt-ii": {
    slug: "hi-opt-ii",
    author: "Lance Humble et Carl Cooper",
    year: 1980,
    tagline: "Niveau 2 avec side-count des As : le roi des déviations.",
    paragraphs: [
      "Le Hi-Opt II, popularisé par Lance Humble et Carl Cooper dans The World's Greatest Blackjack Book (1980), cherche à maximiser l'efficacité de jeu. Ses tags vont jusqu'à ±2 : le 4 et le 5, les cartes les plus favorables au croupier, valent +2, et le 10 vaut −2.",
      "L'As y est neutre (0). C'est un choix délibéré : l'As se comporte comme une carte basse pour certaines décisions de jeu (doubler) et comme une carte haute pour la mise (il fait les blackjacks). En le sortant du compte principal, le Hi-Opt II gagne en précision sur les décisions de jeu.",
      "Le revers : le compte principal sous-estime la valeur d'un sabot riche en As. On suit donc les As restants à part (side-count) et on corrige le compte de mise selon l'excédent ou le déficit d'As par rapport à l'attendu. Le compte de jeu, lui, n'est pas corrigé.",
      "C'est un système exigeant, destiné aux joueurs expérimentés qui veulent tirer le maximum des déviations et de l'assurance.",
    ],
    strengths: [
      "Meilleures efficacité de jeu (PE .67) et corrélation d'assurance (IC .91) du comparatif.",
      "Tags à ±2 qui suivent mieux la valeur réelle de chaque carte.",
      "Le side-count des As restaure une bonne qualité de pari.",
    ],
    weaknesses: [
      "Lourd : niveau 2 et side-count des As à tenir en parallèle.",
      "Corrélation de pari brute plus faible (BC .91) sans le side-count.",
      "Index de déviations à apprendre spécifiquement.",
    ],
  },
  "omega-ii": {
    slug: "omega-ii",
    author: "Bryce Carlson",
    year: 1989,
    tagline: "Niveau 2 avec side-count des As, équilibré entre jeu et pari.",
    paragraphs: [
      "L'Omega II, présenté par Bryce Carlson dans Blackjack for Blood, est un proche cousin du Hi-Opt II. Il en reprend la structure (niveau 2, As neutre) mais affine deux valeurs : le 6 vaut +2 et le 9 vaut −1.",
      "Ces deux changements rapprochent les tags de l'effet réel de chaque carte sur l'avantage du joueur. Le résultat est un système qui garde la très bonne efficacité de jeu du Hi-Opt II tout en améliorant un peu sa corrélation de pari.",
      "Comme le Hi-Opt II, l'As est neutre au compte principal ; le side-count des As est indispensable pour bien miser, puisque ce sont les As qui produisent les blackjacks payés 3:2.",
      "L'Omega II s'adresse au même public que le Hi-Opt II : des joueurs prêts à investir dans un apprentissage long pour un gain de précision.",
    ],
    strengths: [
      "Très bon équilibre entre jeu (PE .67) et pari (BC .92).",
      "Tags proches de l'effet réel de chaque carte (6 à +2, 9 à −1).",
      "Le side-count des As corrige la principale faiblesse du compte de mise.",
    ],
    weaknesses: [
      "Complexité comparable au Hi-Opt II : niveau 2 et side-count.",
      "Le 9 à −1 ajoute une valeur à mémoriser par rapport aux systèmes niveau 1.",
      "Corrélation d'assurance (IC .85) en retrait du Hi-Opt II.",
    ],
  },
  "wong-halves": {
    slug: "wong-halves",
    author: "Stanford Wong",
    year: 1975,
    tagline: "Tags fractionnaires : le meilleur indicateur de mise.",
    paragraphs: [
      "Le Wong Halves, publié par Stanford Wong dans Professional Blackjack (1975), vise la précision maximale au pari. Ses tags reproduisent de près l'effet de chaque carte sur l'avantage du joueur, au prix de valeurs fractionnaires.",
      "Le 5, la carte la plus favorable au croupier, vaut +1,5. Le 2 et le 7 valent +0,5, le 9 vaut −0,5, le 3, le 4 et le 6 valent +1, le 10 et l'As valent −1. Le système est équilibré : un jeu complet totalise 0.",
      "En pratique, les joueurs humains doublent souvent toutes les valeurs pour éviter les fractions, ce qui revient à un compte de niveau 3. La charge mentale reste la plus élevée du comparatif.",
      "Point essentiel pour cette étude : en simulation, ce coût humain disparaît. Le moteur additionne des demi-points sans jamais se tromper ; les résultats montrent donc ce que vaut l'information du Wong Halves, pas ce qu'un joueur réel en tirerait.",
    ],
    strengths: [
      "Meilleure corrélation de pari du comparatif (BC .99).",
      "Tags très proches de l'effet réel de chaque carte.",
      "Équilibré : true count direct, sans side-count.",
    ],
    weaknesses: [
      "Le plus dur mentalement : valeurs fractionnaires ou doublées.",
      "Efficacité de jeu moyenne (PE .56).",
      "Corrélation d'assurance la plus faible du comparatif (IC .72).",
    ],
  },
  zen: {
    slug: "zen",
    author: "Arnold Snyder",
    year: 1983,
    tagline: "Le tout-terrain : niveau 2, As compté, sans side-count.",
    paragraphs: [
      "Le Zen Count, présenté par Arnold Snyder dans The Blackbook of Blackjack (1983), se veut un tout-terrain. C'est un système de niveau 2 qui compte l'As (−1) : il n'a donc pas besoin de side-count, contrairement au Hi-Opt II et à l'Omega II.",
      "Ses valeurs : 2 et 3 à +1, 4, 5 et 6 à +2, 7 à +1, 8 et 9 à 0, 10 à −2, As à −1. Le compte est équilibré.",
      "Ce compromis lui donne une efficacité de jeu élevée (PE .63) tout en conservant une très bonne corrélation de pari (BC .96), sans la charge d'un compte annexe.",
      "Le Zen s'adresse aux joueurs qui veulent dépasser le Hi-Lo sans aller jusqu'aux systèmes à side-count.",
    ],
    strengths: [
      "Excellent compromis jeu / pari (PE .63, BC .96).",
      "Pas de side-count : l'As est déjà dans le compte.",
      "Bonne corrélation d'assurance (IC .85).",
    ],
    weaknesses: [
      "Niveau 2 : plus exigeant que le Hi-Lo.",
      "Ni le meilleur au pari (Wong Halves) ni le meilleur au jeu (Hi-Opt II).",
      "Index de déviations à apprendre spécifiquement.",
    ],
  },
};
