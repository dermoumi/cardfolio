import type { FC, PropsWithChildren } from "react";

import { useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";

import Button from "../Button";

export type BackButtonProps = PropsWithChildren<{
  to?: string;
  from?: string;
}>;

/**
 * Checks if the given history state is the initial state.
 */
function isInitialHistoryState(): boolean {
  // Sometimes, TSR might not have set the initial history state
  const { state } = window.history;
  return state === null || state.__TSR_index === 0;
}

/**
 * Get the parent path of the given path
 */
function getParentPath(path: string): string {
  const segments = path.split("/").filter(Boolean);
  if (segments.length <= 1) return "/";
  segments.pop();
  return `/${segments.join("/")}`;
}

/**
 * A button that navigates back to the previous page.
 * If there is no history, it navigates to the specified path or the parent path.
 */
const BackButton: FC<BackButtonProps> = ({
  from,
  to = getParentPath(window.location.pathname),
  children,
}) => {
  const navigate = useNavigate();

  const handleClick = useCallback(() => {
    if (!isInitialHistoryState()) {
      window.history.back();
      return;
    }

    navigate({ from, to, replace: true });
  }, [navigate, from, to]);

  return <Button onClick={handleClick}>{children ?? "← Back"}</Button>;
};

export default BackButton;
