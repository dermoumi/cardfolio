import { createFileRoute } from "@tanstack/react-router";
import { useMemo, useState } from "react";

import YgoCardListView from "@/components/YgoCardListView";
import { useDebounce } from "@/hooks/useDebounce";
import { useYgoCards } from "@/hooks/useYgoCards";

export const Route = createFileRoute("/cards")({
  component: CardsPage,
});

function CardsPage() {
  const [pageLimit] = useState(50);
  const [filters, setFilters] = useState({});
  const debouncedFilters = useDebounce(filters, 750);

  const { data, fetchNextPage, hasNextPage, isFetchingNextPage } = useYgoCards(
    debouncedFilters,
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
      filters={filters}
      setFilters={setFilters}
    />
  );
}
