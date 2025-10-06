import type { MouseEventHandler, PropsWithChildren } from "react";

import classNames from "classnames";
import { useCallback, useMemo } from "react";

import styles from "./ListItem.module.css";
import ListItemAction from "./ListItemAction";

export type ListItemProps = PropsWithChildren<{
  onClick?: () => void;
}>;

const ListItem = ({ children, onClick }: ListItemProps) => {
  const { actions, content } = useMemo(() => {
    const childrenList = Array.isArray(children) ? children : [children];

    return childrenList.reduce((acc, child) => {
      if (typeof child === "object" && child?.type === ListItemAction) {
        acc.actions.push(child);
      } else {
        acc.content.push(child);
      }

      return acc;
    }, { actions: [], content: [] });
  }, [children]);

  const handleClick: MouseEventHandler = useCallback((e) => {
    e.stopPropagation();
    onClick?.();
  }, [onClick]);

  return (
    <li
      className={classNames(styles.listItem, { [styles.clickable]: onClick })}
      onClick={handleClick}
    >
      <div className={styles.content}>
        {content}
      </div>
      {actions.length > 0 && <div className={styles.actions}>{actions}</div>}
    </li>
  );
};

export default ListItem;
