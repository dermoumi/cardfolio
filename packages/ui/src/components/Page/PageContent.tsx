import type { FC, PropsWithChildren } from "react";

import styles from "./PageContent.module.css";

export type PageContentProps = PropsWithChildren;

const PageContent: FC<PageContentProps> = ({ children }) => {
  return <div className={styles.pageContent}>{children}</div>;
};

export default PageContent;
