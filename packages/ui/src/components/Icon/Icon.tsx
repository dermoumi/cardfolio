import type { FC } from "react";

import { ArrowLeft, Network } from "lucide-react";

const ICON_MAP = {
  network: Network,
  arrowLeft: ArrowLeft,
};

export type IconName = keyof typeof ICON_MAP;

export type IconProps = {
  name: IconName;
  label?: string;
};

const Icon: FC<IconProps> = ({ name, label }) => {
  const IconComponent = ICON_MAP[name];

  return <IconComponent aria-label={label} />;
};

export default Icon;
