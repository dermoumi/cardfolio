import type { FC, PropsWithChildren } from "react";

import { useState } from "react";

import BackButton from "./BackButton";
import { HeaderContext } from "./HeaderContext";
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
  const [toolbarRef, setToolbarRef] = useState<HTMLDivElement | null>(null);

  return (
    <HeaderContext.Provider value={{ toolbarRef }}>
      <main className={styles.page}>
        <header className={styles.pageHeader}>
          <h2 className={styles.pageTitle}>{title}</h2>
          <div className={styles.pageToolbar} ref={setToolbarRef} />
        </header>
        {children}
      </main>
    </HeaderContext.Provider>
  );
};

Page.Content = PageContent;
Page.Toolbar = PageToolbar;
Page.ToolbarSpacer = PageToolbarSpacer;
Page.BackButton = BackButton;

export default Page;
