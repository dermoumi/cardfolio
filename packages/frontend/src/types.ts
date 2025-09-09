/**
 * A Yu-Gi-Oh! card as returned by the backend API.
 */
export type YgoCard = {
  id: number;
  name: string;
  description: string;
  kind: "monster" | "spell" | "trap";
  password: string | null;
  konamiId: number | null;
  tcgDate: string | null;
  ocgDate: string | null;
  monsterKind?: "normal" | "effect" | "ritual" | "fusion" | "synchro" | "xyz" | "link";
  monsterRace: string | null;
  monsterAtk: number | null;
  monsterDef: number | null;
  monsterLevel: number | null;
  spellKind?: "normal" | "field" | "equip" | "quick-play" | "ritual";
  trapKind?: "normal" | "continuous" | "counter";
};

/**
 * Filters that can be applied when querying for Yu-Gi-Oh! cards.
 */
export type YgoCardFilters = Record<string, string>;
