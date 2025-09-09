import type { YgoCard } from "@/types";
import type { FC } from "react";

import YgoCardItem from "../YgoCardItem";
import styles from "./YgoCardGrid.module.css";

export type YgoCardGridProps = {
  cards: Array<YgoCard>;
};

const YgoCardGrid: FC<YgoCardGridProps> = ({ cards }) => {
  return (
    <div className={styles.ygoCardGrid}>
      {cards.map((card) => <YgoCardItem card={card} key={card.id} />)}
    </div>
  );
};

export default YgoCardGrid;
