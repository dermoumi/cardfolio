import type { MouseEventHandler, PropsWithChildren } from "react";

import classNames from "classnames";
import { useCallback } from "react";

import useSplit from "../../hooks/useSplit";
import Avatar from "../Avatar";
import styles from "./ListItem.module.css";
import ListItemAction from "./ListItemAction";

export type ListItemProps = PropsWithChildren<{
  onClick?: () => void;
  avatar?: React.ReactNode;
}>;

const ListItem = ({ children, onClick }: ListItemProps) => {
  const [avatar, actions, content] = useSplit(children, Avatar, ListItemAction);
  console.log(actions);

  const handleClick: MouseEventHandler = useCallback((e) => {
    e.stopPropagation();
    onClick?.();
  }, [onClick]);

  return (
    <li
      className={classNames(styles.listItem, { [styles.clickable]: onClick })}
      onClick={handleClick}
    >
      {avatar.length > 0 && <div className={styles.avatar}>{avatar}</div>}
      <div className={styles.content}>{content}</div>
      {actions.length > 0 && <div className={styles.actions}>{actions}</div>}
    </li>
  );
};

export default ListItem;
