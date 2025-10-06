import type { FC, MouseEventHandler } from "react";
import type { IconName } from "../Icon";

import { useCallback } from "react";

import Button from "../Button";

export type ListItemActionProps = {
  label: string;
  icon: IconName;
  onClick: () => void;
};

const ListItemAction: FC<ListItemActionProps> = ({ label, icon, onClick }) => {
  const handleClick: MouseEventHandler = useCallback((e) => {
    e.stopPropagation();
    onClick();
  }, [onClick]);

  return <Button onClick={handleClick} icon={icon} label={label} variant="secondary" size="sm" />;
};

export default ListItemAction;
