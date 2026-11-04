# Méthodologie

Ce document décrit ce que simule le moteur, comment les résultats sont agrégés et ce qu'ils valent. Tous les chiffres cités proviennent des sorties du projet (`dashboard/public/data/*.json`, tests de validation) ; les sources externes sont indiquées.

## 1. Règles simulées

| Règle | Valeur |
| --- | --- |
| Jeux | 6 |
| Pénétration | 75 % (4,5 jeux sur 6), carte de coupe à `floor(312 × 0,75)` = 234 |
| Croupier | reste sur soft 17 (S17) |
| Blackjack | payé 3:2 |
| Double | sur deux cartes quelconques, après split autorisé (DAS) |
| Split | jusqu'à 4 mains, pas de resplit des As, une seule carte par As splitté |
| Abandon | tardif (late surrender), après le peek |
| Peek | le croupier vérifie le blackjack avec un As ou un 10 visible ; le joueur ne perd que sa mise initiale |
| Assurance | proposée face à un As, décidée par le compte (jamais en basic strategy) |

Le remélange a lieu en fin de main, dès que la carte de coupe est atteinte, jamais au milieu d'une main. Les compteurs repartent alors de leur IRC.

La basic strategy est la stratégie totale 4-8 jeux de Wizard of Odds pour S17, DAS et late surrender ([charts](https://wizardofodds.com/games/blackjack/strategy/4-decks/)), avec les replis standards quand une action est interdite (Dh → tirer, Ds → rester, Rh/Rs/Rp → tirer/rester/splitter). Deux cellules diffèrent volontairement de l'énoncé initial du projet, qui suivait les charts H17 : en S17 6 jeux, 11 contre As se tire et A,7 contre 2 reste (les tests vérifient les deux variantes S17 et H17).

## 2. Protocole Common Random Numbers (CRN)

* Les seeds de run sont dérivées une seule fois de `master_seed` par SplitMix64 ; une seed correspond à un run de `n_hands_per_run` mains.
* Le k-ième sabot d'un run est mélangé (Fisher-Yates) par un Xoshiro256++ semé par `(seed du run, k)`. Le k-ième sabot est donc identique pour les six systèmes et pour toutes les configs ; seul l'instant exact du remélange (fin de main) peut décaler la distribution d'une ou deux cartes.
* Les runs sont répartis par `rayon` dans un pool local, collectés dans l'ordre des seeds, puis réduits séquentiellement. Le résultat est identique au bit près quel que soit le nombre de threads (test `same_seeds_same_results_regardless_of_threads`).

Les écarts entre systèmes sont donc mesurés sur les mêmes cartes : la variance commune s'annule en grande partie.

## 3. Comptage

### True count et arrondi

Pour les systèmes équilibrés, `true count = running count / jeux restants`. Les jeux restants sont calculés **exactement** (cartes restantes / 52), alors qu'un joueur humain les estime à la demi-jeu près : la simulation est donc un peu optimiste sur ce point.

Pour les décisions de jeu, le true count est **tronqué vers −∞** (`floor`) : un TC de +2,9 compte comme +2 et un TC de −0,1 comme −1. C'est la convention de la plupart des tables d'index (une déviation « à +3 » ne se joue qu'à partir de +3,0). Les bins exportés dans `tc_distribution.json` utilisent la même troncature.

### KO

Le KO n'est pas équilibré (+4 par jeu). Son IRC vaut `4 × (1 − n_decks)`, soit −20 pour 6 jeux, et son pivot vaut +4.

* **Mise** : le compte de mise est le running count brut, sans division. Les bins de `tc_distribution.json` sont donc, pour KO, des bins de running count.
* **Déviations** : le running count est converti en compte équilibré exact, `(rc − IRC − 4 × jeux joués) / jeux restants`, puis comparé aux index Hi-Lo mis à l'échelle (section 4). Comme KO = Hi-Lo + 7, cette conversion donne un true count « Hi-Lo + 7 » ; c'est une approximation des index KO publiés.

### Side-count des As (Hi-Opt II, Omega II)

L'As vaut 0 dans le compte principal. Le moteur suit les As sortis et calcule l'excédent d'As restants `As restants − 4 × jeux restants`. Le compte de mise vaut :

```
compte de mise = (running count + ace_factor × excédent d'As) / jeux restants
```

avec `ace_factor = 2` par défaut (paramétrable). Le compte de jeu, utilisé pour les déviations et l'assurance, n'inclut pas cette correction.

## 4. Déviations et mise à l'échelle des index

Les index sont ceux de *Blackjack Attack* (D. Schlesinger) pour Hi-Lo, 6 jeux, S17 : Illustrious 18 (assurance comprise) et Fab 4 (abandons). Chaque règle encode son sens : `TC ≥ index` (ex. 12 contre 3 : rester à partir de +2) ou `TC < index` (ex. 13 contre 2 : tirer en dessous de −1). Pour le Fab 4, l'abandon est joué si `TC ≥ index` et refusé sinon, même quand la basic strategy abandonnerait (15 contre 10).

Pour les autres systèmes, chaque index est multiplié par le ratio des écarts-types des tags sur un jeu complet, puis arrondi à l'entier :

| Système | Facteur vs Hi-Lo |
| --- | --- |
| Hi-Lo | 1,00 |
| KO | 1,05 |
| Hi-Opt II | 1,67 |
| Omega II | 1,79 |
| Wong Halves | 1,05 |
| Zen Count | 1,79 |

C'est une approximation : les index publiés propres à chaque système peuvent différer d'une unité, et certains systèmes gagneraient à jouer davantage de déviations que les 22 retenues. Le comparatif mesure donc surtout la qualité du signal de **mise**, et sous-estime probablement l'avantage des systèmes à forte efficacité de jeu (Hi-Opt II, Omega II).

## 5. Modèles de pari

### Modèle A : unités, bankroll infinie

```
mise = clamp(1, spread, floor(compte de mise / facteur − ramp_offset))
```

avec `ramp_offset = 1` (« TC − 1 unités ») pour toutes les configs publiées. Le compte de mise est ramené à l'échelle Hi-Lo par le même facteur qu'en section 4, pour que tous les systèmes équilibrés suivent la même rampe. Pour KO, la rampe démarre au pivot : `mise = clamp(1, spread, floor(rc − 4))`.

Sorties : avantage (gain total / mises initiales totales), EV pour 100 mains, écart-type par main, N₀, risque de ruine, fraction de Kelly, mise moyenne, taux de mains gagnées.

### Modèle B : bankroll finie

Bankroll initiale de 1 000 unités, mise minimale de 1 unité. Politiques :

* `flat_pct` : `pct × B_t`, indépendamment du compte (10 % dans la config `s15-flat10`, mise maximale 1 000 unités) ;
* `fractional_kelly` : `k × max(0, edge(TC)) / variance × B_t`, avec `k = 0,5` (demi-Kelly), bornée par la mise minimale et par une mise maximale égale au spread de la config (15 ou 5 unités). `edge(TC)` est l'avantage par unité misée mesuré par le modèle A dans le bin du compte (bins de moins de 100 000 mains remplacés par le bin fiable le plus proche) ; la variance est celle du résultat par unité misée ;
* `count_proportional` : unités de la rampe × fraction de bankroll (implémentée, non publiée).

Un run est ruiné dès que `B_t` passe sous la mise minimale ; sa bankroll est alors figée. La bankroll ne descend jamais sous 0 : sans fonds, le joueur ne peut pas doubler ni splitter. Le drawdown d'un run est le pire repli relatif depuis un plus haut.

L'horizon du modèle B est de 100 000 mains (Kelly) et de 5 000 mains pour `s15-flat10`, dont la ruine survient en quelques centaines de mains et serait invisible avec un échantillonnage grossier.

## 6. Formules

* **N₀** : `(σ / EV)²`, nombre de mains pour que l'espérance cumulée égale un écart-type. Non défini (`null`) si l'espérance est négative.
* **Risque de ruine** : `exp(−2 · EV · B / σ²)` avec EV et σ par main en unités et `B = 1 000` unités. Vaut 1 si l'espérance est négative.
* **Fraction de Kelly** : `EV / σ²`, taille d'unité optimale en fraction de la bankroll pour la rampe donnée.

## 7. Résultats de validation

Les tests lourds se lancent avec `cargo test --release -- --ignored --nocapture`.

### Basic strategy seule (bloquant) — validée

Référence : [House Edge Calculator de Wizard of Odds](https://wizardofodds.com/games/blackjack/calculator/), règles de la section 1, avantage maison de 0,330512 %, plus la correction « basic strategy with cut card » de +0,0231 % pour 6 jeux, soit **−0,3536 %** d'avantage joueur attendu.

| Mesure | Valeur | Tolérance | Statut |
| --- | --- | --- | --- |
| Avantage (200 runs × 125 000 mains = 25 M mains) | −0,3580 % (±0,0228 % à 1 σ) | −0,3536 % ± 0,05 % | ✅ |
| Écart-type par main | 1,1409 | 1,15 ± 0,02 | ✅ |

La baseline du lancement complet (`baseline.json`, 2 milliards de mains) donne −0,3551 %, à 0,0015 point de la référence.

### Hi-Lo 1-5, pénétration 4/6 — validée

400 runs × 100 000 mains, I18 + Fab 4, rampe « TC − 1 » : **+0,0732 %**, pour une valeur de référence de l'ordre de +0,16 % (tolérance ±0,15 %). Le test passe, mais la mesure est dans la moitié basse de la fourchette.

### Hi-Lo 1-15, pénétration 4,5/6 — écart documenté

Valeur de référence annoncée : environ +0,83 %. Mesure : **+0,3146 %**, soit un écart de −0,52 point. Le test vérifie seulement que l'avantage est significativement positif et supérieur à celui du spread 1-5 ; il ne masque pas l'écart en élargissant la tolérance.

Le moteur n'est pas en cause : en mise plate avec I18 + Fab 4, Hi-Lo donne −0,2655 %, et l'avantage croît d'environ +0,55 % par point de true count, conformément à la littérature. L'écart vient de la rampe imposée (une unité par point de TC au-dessus de l'offset), beaucoup moins agressive que les rampes des tables de référence, qui montent au maximum dès TC +4 ou +5. Balayage de l'offset sur 40 M mains :

| `ramp_offset` | Hi-Lo 1-15, pén. 0,75 | Hi-Lo 1-5, pén. 4/6 |
| --- | --- | --- |
| 1 (publié) | +0,2998 % | +0,0483 % |
| 0 | +0,4793 % | +0,1939 % |
| −1 | +0,6158 % | +0,3181 % |
| −2 | +0,6084 % | +0,3158 % |

Même la rampe la plus agressive plafonne vers +0,62 %. Atteindre +0,83 % demanderait une rampe en marches (saut au maximum à un TC donné) et probablement le wonging. La rampe publiée est conservée telle qu'énoncée, identique pour tous les systèmes, et l'écart est assumé.

## 8. Limites

* Les index des systèmes autres que Hi-Lo sont des transpositions (section 4), pas les index publiés de chaque système.
* Les jeux restants sont exacts et le comptage sans erreur : un joueur réel fait des erreurs, surtout avec les systèmes de niveau 2 et 3, et estime les jeux restants au demi-jeu.
* La rampe linéaire « TC − 1 » est volontairement simple et identique pour tous ; elle n'exploite pas tout le spread 1-15.
* Le modèle de casino est idéal : pas de contre-mesures, pas de limite de temps, pénétration fixe, une seule place à la table.
* Ce projet est une étude de simulation, pas une méthode de jeu.
