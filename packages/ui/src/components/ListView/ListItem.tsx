import type { MouseEventHandler, PropsWithChildren } from "react";

import classNames from "classnames";
import { useCallback } from "react";

import useSplit from "../../hooks/useSplit";
import styles from "./ListItem.module.css";
import ListItemAction from "./ListItemAction";

export type ListItemProps = PropsWithChildren<{
  onClick?: () => void;
}>;

const ListItem = ({ children, onClick }: ListItemProps) => {
  const [actions, content] = useSplit(children, ListItemAction);

  const handleClick: MouseEventHandler = useCallback((e) => {
    e.stopPropagation();
    onClick?.();
  }, [onClick]);

  return (
    <li
      className={classNames(styles.listItem, { [styles.clickable]: onClick })}
      onClick={handleClick}
    >
      <div className={styles.content}>{content}</div>
      {actions.length > 0 && <div className={styles.actions}>{actions}</div>}
    </li>
  );
};

export default ListItem;
