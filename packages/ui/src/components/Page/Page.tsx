import type { FC, PropsWithChildren } from "react";

import BackButton from "./BackButton";
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
  BackButton: typeof BackButton;
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
Page.BackButton = BackButton;

export default Page;
