import type { FC, PropsWithChildren } from "react";

import styles from "./YgoCardGrid.module.css";

export type YgoCardGridProps = PropsWithChildren;

const YgoCardGrid: FC<YgoCardGridProps> = ({ children }) => {
  return <div className={styles.ygoCardGrid}>{children}</div>;
};

export default YgoCardGrid;
