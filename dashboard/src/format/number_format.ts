/**
 * Formatage des nombres en français (virgule décimale, espace des milliers).
 */

const LOCALE = "fr-FR";

const integerFormat = new Intl.NumberFormat(LOCALE, { maximumFractionDigits: 0 });

/**
 * Formate un nombre avec un nombre fixe de décimales.
 *
 * @param value - Valeur à formater.
 * @param digits - Nombre de décimales (défaut : 2).
 * @returns Chaîne formatée, ex. « 1,15 ».
 */
export function formatNumber(value: number, digits = 2): string {
  return new Intl.NumberFormat(LOCALE, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  }).format(value);
}

/**
 * Formate une valeur déjà exprimée en pourcentage (ex. `0.31` → « 0,31 % »).
 *
 * @param valuePct - Valeur en pourcentage.
 * @param digits - Nombre de décimales (défaut : 2).
 * @param signed - Affiche le signe « + » pour les valeurs positives.
 * @returns Chaîne formatée.
 */
export function formatPercent(valuePct: number, digits = 2, signed = false): string {
  const formatted = new Intl.NumberFormat(LOCALE, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
    signDisplay: signed ? "exceptZero" : "auto",
  }).format(valuePct);
  return `${formatted} %`;
}

/**
 * Formate une fraction en pourcentage (ex. `0.0312` → « 3,12 % »).
 *
 * @param fraction - Valeur dans [0, 1].
 * @param digits - Nombre de décimales (défaut : 2).
 * @returns Chaîne formatée.
 */
export function formatFractionAsPercent(fraction: number, digits = 2): string {
  return formatPercent(100 * fraction, digits);
}

/**
 * Formate un entier avec séparateur de milliers (ex. « 226 001 »).
 *
 * @param value - Valeur à arrondir puis formater.
 * @returns Chaîne formatée.
 */
export function formatInteger(value: number): string {
  return integerFormat.format(Math.round(value));
}

/**
 * Formate N₀ : entier avec séparateur de milliers, tiret si non défini.
 *
 * @param n0 - Valeur de N₀ ou `null` (espérance négative).
 * @returns Chaîne formatée.
 */
export function formatN0(n0: number | null): string {
  return n0 === null ? "—" : formatInteger(n0);
}

/**
 * Formate un grand nombre de façon compacte (ex. « 26,1 milliards »).
 *
 * @param value - Valeur à formater.
 * @returns Chaîne formatée.
 */
export function formatCompact(value: number): string {
  return new Intl.NumberFormat(LOCALE, {
    notation: "compact",
    compactDisplay: "long",
    maximumFractionDigits: 1,
  }).format(value);
}

/**
 * Formate un tag de comptage (ex. `0.5` → « +0,5 », `-2` → « −2 », `0` → « 0 »).
 *
 * @param tag - Valeur du tag.
 * @returns Chaîne avec signe explicite et vrai signe moins.
 */
export function formatTag(tag: number): string {
  const formatted = new Intl.NumberFormat(LOCALE, {
    maximumFractionDigits: 2,
    signDisplay: "exceptZero",
  }).format(tag);
  return formatted.replace("-", "−");
}
