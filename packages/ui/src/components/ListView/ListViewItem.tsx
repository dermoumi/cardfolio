import type { MouseEventHandler, PropsWithChildren, ReactNode } from "react";

import classNames from "classnames";

import styles from "./ListViewItem.module.css";

export type ListViewItemProps = PropsWithChildren<{
  actions?: ReactNode;
  onClick?: MouseEventHandler;
}>;

const ListViewItem = ({ children, actions, onClick }: ListViewItemProps) => {
  return (
    <li
      className={classNames(styles.listViewItem, { [styles.clickable]: onClick })}
      onClick={onClick}
    >
      <div className={styles.content}>
        {children}
      </div>
      {actions && <div className={styles.actions}>{actions}</div>}
    </li>
  );
};

export default ListViewItem;
