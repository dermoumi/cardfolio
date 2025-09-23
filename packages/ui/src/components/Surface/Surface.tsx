import type { FC, PropsWithChildren } from "react";

import classNames from "classnames";

import styles from "./Surface.module.css";

export type SurfaceProps = PropsWithChildren<{
  variant?: "soft" | "outlined" | "transparent";
}>;

const VARIANT_MAP = {
  soft: styles.soft,
  outlined: styles.outlined,
  transparent: styles.transparent,
} as const;

const Surface: FC<SurfaceProps> = ({ children, variant = "soft" }) => {
  return <div className={classNames(styles.surface, VARIANT_MAP[variant])}>{children}</div>;
};

export default Surface;
