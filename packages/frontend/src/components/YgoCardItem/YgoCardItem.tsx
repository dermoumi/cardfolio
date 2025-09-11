import type { YgoCard } from "@/types";
import type { FC } from "react";

import styles from "./YgoCardItem.module.css";

export type YgoCardItemProps = {
  card: YgoCard;
};

const YgoCardItem: FC<YgoCardItemProps> = ({ card }) => {
  const imgSrc = `/api/v1/ygo/cards/${card.id}/image`;

  return (
    <div className={styles.ygoCardItem}>
      <img src={imgSrc} alt={card.name} />
    </div>
  );
};

export default YgoCardItem;
