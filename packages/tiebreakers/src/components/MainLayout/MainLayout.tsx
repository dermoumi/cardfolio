import type { FC, PropsWithChildren } from "react";

import { Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

export type MainLayoutProps = PropsWithChildren;

const MainLayout: FC<MainLayoutProps> = () => {
  return (
    <>
      <Outlet />
      <TanStackRouterDevtools position="top-right" />
    </>
  );
};

export default MainLayout;
