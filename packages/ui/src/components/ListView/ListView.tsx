import type { FC, PropsWithChildren } from "react";

import styles from "./ListView.module.css";
import ListViewItem from "./ListViewItem";

export type ListViewProps = PropsWithChildren;

type ListviewComponent = FC<ListViewProps> & {
  Item: typeof ListViewItem;
};

const ListView: ListviewComponent = ({ children }) => {
  return <ul className={styles.listView}>{children}</ul>;
};

ListView.Item = ListViewItem;

export default ListView;
