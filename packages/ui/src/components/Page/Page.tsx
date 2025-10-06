import type { FC, PropsWithChildren } from "react";

import { useElementScrollRestoration } from "@tanstack/react-router";
import classNames from "classnames";
import { useEffect, useState } from "react";

import BackButton from "./BackButton";
import { PageContext } from "./context";
import styles from "./Page.module.css";
import PageHeader from "./PageHeader";

export type PageProps = PropsWithChildren;

type PageComponent = FC<PageProps> & {
  Header: typeof PageHeader;
  BackButton: typeof BackButton;
};

const Page: PageComponent = ({ children }) => {
  const [fabsRegistered, setFabsRegistered] = useState(0);

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

  const registerFab = () => {
    setFabsRegistered((count) => count + 1);
  };

  const unregisterFab = () => {
    setFabsRegistered((count) => Math.max(0, count - 1));
  };

  return (
    <PageContext.Provider value={{ registerFab, unregisterFab }}>
      <main className={classNames(styles.page, { [styles.hasFab]: fabsRegistered > 0 })}>
        <div className={styles.pageContent}>
          {children}
        </div>
      </main>
    </PageContext.Provider>
  );
};

Page.Header = PageHeader;
Page.BackButton = BackButton;

export default Page;
