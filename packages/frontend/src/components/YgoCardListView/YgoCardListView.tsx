import type { YgoCard, YgoCardFilters } from "@/types";
import type { FC } from "react";

import { Button } from "@cardfolio/ui";

import YgoCardGrid from "../YgoCardGrid";
import YgoCardListFilter from "./YgoCardListFilter";
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
  return (
    <div className={styles.cardListView}>
      <YgoCardListFilter filters={filters} setFilters={setFilters} />
      <div className={styles.gridContainer}>
        <YgoCardGrid cards={cards} />
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
