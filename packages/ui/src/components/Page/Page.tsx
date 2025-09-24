import type { FC, PropsWithChildren } from "react";

import { useElementScrollRestoration } from "@tanstack/react-router";
import { useEffect, useState } from "react";

import BackButton from "./BackButton";
import { HeaderContext } from "./HeaderContext";
import styles from "./Page.module.css";
import PageToolbar from "./PageToolbar";
import PageToolbarSpacer from "./PageToolbarSpacer";

export type PageProps = PropsWithChildren<{
  title?: string;
}>;

type PageComponent = FC<PageProps> & {
  Toolbar: typeof PageToolbar;
  ToolbarSpacer: typeof PageToolbar;
  BackButton: typeof BackButton;
};

const Page: PageComponent = ({ title, children }) => {
  const [toolbarRef, setToolbarRef] = useState<HTMLDivElement | null>(null);

  // Restore scroll position on mount, after first render
  const scrollEntry = useElementScrollRestoration({
    getElement: () => window,
  });
  useEffect(() => {
    const scrollY = scrollEntry?.scrollY;
    if (scrollY === undefined) return;

    // This is needed since the initial render doesn't have toolbar elements
    setTimeout(() => window.scrollTo(0, scrollY));
  }, [scrollEntry]);

  return (
    <main className={styles.page}>
      <header className={styles.pageHeader}>
        <h2 className={styles.pageTitle}>{title}</h2>
        <div className={styles.pageToolbar} ref={setToolbarRef} />
      </header>
      <div className={styles.pageContent}>
        <HeaderContext.Provider value={{ toolbarRef }}>
          {children}
        </HeaderContext.Provider>
      </div>
    </main>
  );
};

Page.Toolbar = PageToolbar;
Page.ToolbarSpacer = PageToolbarSpacer;
Page.BackButton = BackButton;

export default Page;
