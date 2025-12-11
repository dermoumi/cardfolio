import type { PropsWithChildren } from "react";

import styles from "./PageContent.module.css";

export type PageContentProps = PropsWithChildren;

const PageContent = ({ children }: PageContentProps) => {
  return <div className={styles.content}>{children}</div>;
};

export default PageContent;
