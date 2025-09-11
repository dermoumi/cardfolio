import type { YgoCard } from "@/types";

import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import YgoCardGrid from ".";

describe("YgoCardGrid", () => {
  const mockCards: Array<YgoCard> = [
    {
      "id": 15,
      "updatedAt": "2025-09-03T21:54:04.817564Z",
      "name": "7 Colored Fish",
      "description": "A rare rainbow fish that has never been caught by mortal man.",
      "kind": "monster",
      "password": "23771716",
      "konamiId": 4446,
      "tcgDate": "2002-06-26",
      "ocgDate": "2000-01-27",
      "monsterKind": "normal",
      "monsterAttribute": "water",
      "monsterRace": "fish",
      "monsterAtk": 1800,
      "monsterDef": 800,
      "monsterLevel": 4,
    },
    {
      "id": 10,
      "updatedAt": "2025-09-03T21:54:02.391850Z",
      "name": "1st Movement Solo",
      "description":
        "If you control no monsters: Special Summon 1 Level 4 or lower \"Melodious\" monster from your hand or Deck. You can only activate 1 \"1st Movement Solo\" per turn. You cannot Special Summon monsters during the turn you activate this card, except \"Melodious\" monsters.",
      "kind": "spell",
      "password": "44256816",
      "konamiId": 11391,
      "tcgDate": "2014-11-06",
      "ocgDate": "2014-07-19",
      "spellKind": "normal",
    },
  ];

  it("matches snapshot", () => {
    const { asFragment } = render(<YgoCardGrid cards={mockCards} />);
    expect(asFragment()).toMatchSnapshot();
  });
});
