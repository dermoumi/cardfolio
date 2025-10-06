import type { FC, PropsWithChildren } from "react";

import ListItem from "./ListItem";
import ListItemAction from "./ListItemAction";
import styles from "./ListView.module.css";

export type ListViewProps = PropsWithChildren;

type ListviewComponent = FC<ListViewProps> & {
  Item: typeof ListItem;
  Action: typeof ListItemAction;
};

const ListView: ListviewComponent = ({ children }) => {
  return <ul className={styles.listView}>{children}</ul>;
};

ListView.Item = ListItem;
ListView.Action = ListItemAction;

export default ListView;
