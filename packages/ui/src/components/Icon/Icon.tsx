import type { FC } from "react";

import { ArrowLeft, CalendarClock, Network, Plus, Trash, X } from "lucide-react";

const ICON_MAP = {
  network: Network,
  arrowLeft: ArrowLeft,
  calendarClock: CalendarClock,
  plus: Plus,
  x: X,
  trash: Trash,
};

export type IconName = keyof typeof ICON_MAP;

export type IconProps = {
  name: IconName;
  label?: string;
  size?: number;
};

const Icon: FC<IconProps> = ({ name, label, size = 16 }) => {
  const IconComponent = ICON_MAP[name];

  return <IconComponent aria-label={label} size={size} />;
};

export default Icon;
