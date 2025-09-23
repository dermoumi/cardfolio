import type { PropsWithChildren } from "react";

import styles from "./ListViewItem.module.css";

export type ListViewItemProps = PropsWithChildren;

const ListViewItem = ({ children }: ListViewItemProps) => {
  return <li className={styles.listViewItem}>{children}</li>;
};

export default ListViewItem;
