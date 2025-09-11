import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import YgoCardItem from ".";

describe("YgoCardItem", () => {
  const mockCard = {
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
  } as const;

  it("renders card image with correct src and alt", () => {
    render(<YgoCardItem card={mockCard} />);
    const imgElement: HTMLImageElement = screen.getByAltText("1st Movement Solo");
    expect(imgElement.src).toContain("/api/v1/ygo/cards/10/image");
  });

  it("matches snapshot", () => {
    const { asFragment } = render(<YgoCardItem card={mockCard} />);
    expect(asFragment()).toMatchSnapshot();
  });
});
