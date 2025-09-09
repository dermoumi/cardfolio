import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";

import YgoCardListView from "@/components/YgoCardListView";
import { useYgoCards } from "@/hooks/useYgoCards";

export const Route = createFileRoute("/cards")({
  component: CardsPage,
});

function CardsPage() {
  const [pageLimit] = useState(50);
  const { data, fetchNextPage, hasNextPage, isFetchingNextPage } = useYgoCards(
    {},
    pageLimit,
  );

  const cards = useMemo(() => {
    return data?.pages.flatMap((page) => page.cards) ?? [];
  }, [data]);

  return (
    <YgoCardListView
      cards={cards}
      fetchNextPage={fetchNextPage}
      hasNextPage={hasNextPage}
      isFetchingNextPage={isFetchingNextPage}
    />
  );
}
