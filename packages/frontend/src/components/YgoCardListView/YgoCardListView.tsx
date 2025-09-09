import type { YgoCard } from "@/types";
import type { FC } from "react";

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
};

const YgoCardListView: FC<YgoCardListProps> = (
  { cards, fetchNextPage, hasNextPage, isFetchingNextPage },
) => {
  return (
    <div className={styles.cardListView}>
      <div className={styles.filterBar}>
        <TextInput
          type="search"
          placeholder="Search cards..."
          value={""}
          onChange={() => {
            // No-op for now
          }}
        />
      </div>
      <div className={styles.gridContainer}>
        <YgoCardGrid>
          {cards.map((card) => <YgoCardItem card={card} key={card.id} />)}
        </YgoCardGrid>
        <Button onClick={() => fetchNextPage()} disabled={!hasNextPage || isFetchingNextPage}>
          {isFetchingNextPage ? "Loading..." : hasNextPage ? "Load More" : "No More Cards"}
        </Button>
      </div>
    </div>
  );
};

export default YgoCardListView;
