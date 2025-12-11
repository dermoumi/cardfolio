import { createRootRoute } from "@tanstack/react-router";

import MainLayout from "@/components/MainLayout";

export const Route = createRootRoute({
  component: () => <MainLayout />,
});
