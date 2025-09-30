import { ScreenSizeProvider } from "@cardfolio/ui";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";

import { routeTree } from "./routeTree.gen";

import "./main.css";

// Create a new router instance
const router = createRouter({
  routeTree,
  context: {},
  defaultPreload: "intent",
  scrollRestoration: true,
  defaultStructuralSharing: true,
  defaultPreloadStaleTime: 0,
});

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  // @ts-expect-error: overriding the original type
  type Register = {
    router: typeof router;
  };
}

// Setup react-query client
const queryClient = new QueryClient();

// Set initial theme to light
document.body.dataset.theme = "light";

// Render the app
const rootElement = document.getElementById("app");
if (rootElement && !rootElement.innerHTML) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <StrictMode>
      <ScreenSizeProvider>
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>
      </ScreenSizeProvider>
    </StrictMode>,
  );
}
