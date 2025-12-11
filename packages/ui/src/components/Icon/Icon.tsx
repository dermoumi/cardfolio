import type { FC } from "react";

import { ArrowLeft, CalendarClock, Network, Plus, Save, Trash, X } from "lucide-react";

const ICON_MAP = {
  network: Network,
  arrowLeft: ArrowLeft,
  calendarClock: CalendarClock,
  plus: Plus,
  x: X,
  trash: Trash,
  save: Save,
};

export type IconName = keyof typeof ICON_MAP;

const SIZE_ICONS = {
  xs: 12,
  sm: 14,
  md: 16,
  lg: 20,
  xl: 32,
};

export type IconSize = keyof typeof SIZE_ICONS;

export type IconProps = {
  name: IconName;
  label?: string;
  size?: IconSize;
};

const Icon: FC<IconProps> = ({ name, label, size = "md" }) => {
  const IconComponent = ICON_MAP[name];

  return <IconComponent aria-label={label} size={SIZE_ICONS[size]} />;
};

export default Icon;
