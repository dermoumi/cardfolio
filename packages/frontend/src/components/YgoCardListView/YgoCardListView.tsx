import type { YgoCard, YgoCardFilters } from "@/types";
import type { FC } from "react";

import { useCallback } from "react";

import Button from "../Button";
import TextInput from "../TextInput";
import YgoCardGrid from "../YgoCardGrid";
import YgoCardItem from "../YgoCardItem";
import styles from "./YgoCardListView.module.css";

export type YgoCardListProps = {
  cards: Array<YgoCard>;
  fetchNextPage: () => void;
  hasNextPage?: boolean;
  isFetchingNextPage?: boolean;
  filters: YgoCardFilters;
  setFilters: (filter: YgoCardFilters) => void;
};

const YgoCardListView: FC<YgoCardListProps> = (
  { cards, fetchNextPage, hasNextPage, isFetchingNextPage, filters, setFilters },
) => {
  const setNameFilter = useCallback((name: string) => {
    setFilters({ ...filters, name });
  }, [setFilters, filters]);

  return (
    <div className={styles.cardListView}>
      <div className={styles.filterBar}>
        <TextInput
          type="search"
          placeholder="Search cards..."
          value={filters.name || ""}
          onChange={setNameFilter}
        />
      </div>
      <div className={styles.gridContainer}>
        <YgoCardGrid>
          {cards.map((card) => <YgoCardItem card={card} key={card.id} />)}
        </YgoCardGrid>
        {hasNextPage && (
          <Button onClick={() => fetchNextPage()} disabled={isFetchingNextPage}>
            {isFetchingNextPage ? "Loading..." : "Load More"}
          </Button>
        )}
      </div>
    </div>
  );
};

export default YgoCardListView;
