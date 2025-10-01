import type { FC } from "react";

import styles from "./PageHeader.module.css";

export type PageHeaderProps = {
  title: string;
  backAction?: React.ReactNode;
  actions?: React.ReactNode;
};

const PageHeader: FC<PageHeaderProps> = ({ title, backAction, actions }) => {
  return (
    <header className={styles.header}>
      {backAction && <div className={styles.backAction}>{backAction}</div>}
      <div className={styles.titleContainer}>
        <h2 className={styles.title}>{title}</h2>
        {actions && <div className={styles.actions}>{actions}</div>}
      </div>
    </header>
  );
};

export default PageHeader;
