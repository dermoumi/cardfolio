import type { FC, PropsWithChildren } from "react";

import styles from "./Page.module.css";
import PageContent from "./PageContent";
import PageToolbar from "./PageToolbar";
import PageToolbarSpacer from "./PageToolbarSpacer";

export type PageProps = PropsWithChildren<{
  title?: string;
}>;

type PageComponent = FC<PageProps> & {
  Content: typeof PageContent;
  Toolbar: typeof PageToolbar;
  ToolbarSpacer: typeof PageToolbar;
};

const Page: PageComponent = ({ title, children }) => {
  return (
    <main className={styles.page}>
      {title && (
        <header className={styles.pageHeader}>
          <h2 className={styles.pageTitle}>{title}</h2>
        </header>
      )}
      {children}
    </main>
  );
};

Page.Content = PageContent;
Page.Toolbar = PageToolbar;
Page.ToolbarSpacer = PageToolbarSpacer;

export default Page;
