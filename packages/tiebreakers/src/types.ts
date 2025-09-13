/**
 * Yu-Gi-Oh! card types
 */
type YgoCardKind = "monster" | "spell" | "trap";

/**
 * Yu-Gi-Oh! monster card types
 */
type YgoMonsterKind =
  | "other"
  | "token"
  | "normal"
  | "effect"
  | "fusion"
  | "ritual"
  | "synchro"
  | "xyz"
  | "link";

/**
 * Yu-Gi-Oh! monster subtypes
 */
type YgoMonsterSubtype = "other" | "flip" | "gemini" | "spirit" | "toon" | "tuner" | "union";

/**
 * Yu-Gi-Oh! monster attributes
 */
type YgoMonsterAttribute =
  | "other"
  | "dark"
  | "divine"
  | "earth"
  | "fire"
  | "light"
  | "water"
  | "wind";

/**
 * Yu-Gi-Oh! monster races
 */
type YgoMonsterRace =
  | "other"
  | "aqua"
  | "beast"
  | "beast_warrior"
  | "creator_god"
  | "cyberse"
  | "dinosaur"
  | "divine_beast"
  | "dragon"
  | "fairy"
  | "fiend"
  | "fish"
  | "illusion"
  | "insect"
  | "machine"
  | "plant"
  | "psychic"
  | "pyro"
  | "reptile"
  | "rock"
  | "sea_serpent"
  | "spellcaster"
  | "thunder"
  | "warrior"
  | "winged_beast"
  | "wyrm"
  | "zombie";

/**
 * Yu-Gi-Oh! link markers
 */
export type YgoLinkArrows =
  | "top_left"
  | "top"
  | "top_right"
  | "left"
  | "right"
  | "bottom_left"
  | "bottom"
  | "bottom_right";

/**
 * Yu-Gi-Oh! spell card types
 */
type YgoSpellKind = "other" | "normal" | "continuous" | "field" | "equip" | "ritual" | "quick_play";

/**
 * Yu-Gi-Oh! trap card types
 */
type YgoTrapKind = "other" | "normal" | "continuous" | "counter";

/**
 * Fields common to all Yu-Gi-Oh! cards.
 */
type YgoCardCommonFields = {
  id: number;
  updatedAt: string;
  name: string;
  description: string;
  password?: string;
  konamiId?: number;
  treatedAs?: number;
  tcgDate?: string;
  ocgDate?: string;
  ygoprodeckId?: number;
};

/**
 * Yu-Gi-Oh! monster cards
 */
type YgoCardMonster = YgoCardCommonFields & {
  kind: "monster";
  monsterKind: YgoMonsterKind;
  monsterAttribute: YgoMonsterAttribute;
  monsterRace: YgoMonsterRace;
  monsterSubtypes?: Array<YgoMonsterSubtype>;
  monsterAtk: number;
  monsterDef: number;
  monsterLevel: number;
  monsterPendulumScale?: number;
  monsterPendulumEffect?: string;
  monsterLinkArrows?: Array<YgoLinkArrows>;
};

/**
 * Yu-Gi-Oh! spell cards
 */
type YgoCardSpell = YgoCardCommonFields & {
  kind: "spell";
  spellKind: YgoSpellKind;
};

/**
 * Yu-Gi-Oh! trap cards
 */
type YgoCardTrap = YgoCardCommonFields & {
  kind: "trap";
  trapKind: YgoTrapKind;
};

/**
 * A Yu-Gi-Oh! card as returned by the backend API.
 */
export type YgoCard = YgoCardMonster | YgoCardSpell | YgoCardTrap;

/**
 * Filters that can be applied when querying for Yu-Gi-Oh! cards.
 */
export type YgoCardFilters = {
  name?: string;
  description?: string;
  kind?: YgoCardKind;
  attribute?: Array<YgoMonsterAttribute>;
  race?: Array<YgoMonsterRace>;
  subtype?: Array<YgoMonsterSubtype>;
  atkMin?: number;
  atkMax?: number;
  defMin?: number;
  defMax?: number;
  levelMin?: number;
  levelMax?: number;
  spellKind?: Array<YgoSpellKind>;
  trapKind?: Array<YgoTrapKind>;
};
