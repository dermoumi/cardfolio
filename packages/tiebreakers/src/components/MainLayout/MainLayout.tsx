import type { FC, PropsWithChildren } from "react";

import { Outlet } from "@tanstack/react-router";

export type MainLayoutProps = PropsWithChildren;

const MainLayout: FC<MainLayoutProps> = () => {
  return <Outlet />;
};

export default MainLayout;
