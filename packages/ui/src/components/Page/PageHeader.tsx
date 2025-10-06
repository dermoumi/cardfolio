import type { FC } from "react";

import classNames from "classnames";

import { useScreenSize } from "../../providers/ScreenSizeProvider";
import styles from "./PageHeader.module.css";

const VARIANT_STYLES = {
  normal: styles.normal,
  centered: styles.centered,
};

export type PageHeaderProps = {
  title: string;
  navSlot?: React.ReactNode;
  actions?: React.ReactNode;
  variant?: keyof typeof VARIANT_STYLES;
};

const PageHeader: FC<PageHeaderProps> = ({ title, navSlot, actions, variant = "normal" }) => {
  const { screenSize } = useScreenSize();

  return (
    <header className={classNames(styles.header, VARIANT_STYLES[variant])}>
      {screenSize !== "lg" && <div className={styles.navSlot}>{navSlot}</div>}
      <div className={styles.titleContainer}>
        <h1 className={styles.title}>{title}</h1>
        {actions && <div className={styles.actions}>{actions}</div>}
      </div>
    </header>
  );
};

export default PageHeader;
