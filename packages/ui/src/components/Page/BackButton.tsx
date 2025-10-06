import type { FC } from "react";

import { useNavigate } from "@tanstack/react-router";
import { useCallback } from "react";

import { useScreenSize } from "../../providers/ScreenSizeProvider";
import Button from "../Button";

export type BackButtonProps = {
  to?: string;
  from?: string;
  label?: string;
};

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
  label = "Back",
}) => {
  const { screenSize } = useScreenSize();

  const navigate = useNavigate();

  const handleClick = useCallback(() => {
    if (!isInitialHistoryState()) {
      window.history.back();
      return;
    }

    navigate({ from, to, replace: true });
  }, [navigate, from, to]);

  return (
    <Button onClick={handleClick} icon="arrowLeft" label={label} variant="subtle">
      {screenSize === "lg" && label}
    </Button>
  );
};

export default BackButton;
