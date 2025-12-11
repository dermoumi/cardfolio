import type { FC, MouseEventHandler } from "react";
import type { IconName } from "../Icon";

import { useCallback } from "react";

import { useScreenSize } from "../../providers/ScreenSizeProvider";
import Button from "../Button";

export type ListItemActionProps = {
  label: string;
  icon: IconName;
  onClick: () => void;
};

const ListItemAction: FC<ListItemActionProps> = ({ label, icon, onClick }) => {
  const { screenSize } = useScreenSize();

  const handleClick: MouseEventHandler = useCallback((e) => {
    e.stopPropagation();
    onClick();
  }, [onClick]);

  return (
    <Button onClick={handleClick} icon={icon} label={label} variant="subtle">
      {screenSize !== "sm" && label}
    </Button>
  );
};

export default ListItemAction;
