import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import YgoCardListFilter from "./YgoCardListFilter";

describe("YgoCardListFilter", () => {
  it("matches snapshot", () => {
    const { asFragment } = render(
      <YgoCardListFilter filters={{ name: "test" }} setFilters={vi.fn()} />,
    );
    expect(asFragment()).toMatchSnapshot();
  });
});
