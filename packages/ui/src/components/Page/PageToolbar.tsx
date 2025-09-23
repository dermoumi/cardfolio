import type { FC, PropsWithChildren } from "react";

import styles from "./PageToolbar.module.css";

export type PageToolbarProps = PropsWithChildren;

const PageToolbar: FC<PageToolbarProps> = ({ children }) => {
  return <div className={styles.pageToolbar}>{children}</div>;
};

export default PageToolbar;
