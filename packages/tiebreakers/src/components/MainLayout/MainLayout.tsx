import type { FC, PropsWithChildren } from "react";

import { Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

import styles from "./MainLayout.module.css";

export type MainLayoutProps = PropsWithChildren;

const MainLayout: FC<MainLayoutProps> = () => {
  return (
    <>
      <div className={styles.mainLayout}>
        <main className={styles.content}>
          <Outlet />
        </main>
      </div>
      <TanStackRouterDevtools position="top-right" />
    </>
  );
};

export default MainLayout;
