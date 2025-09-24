import type { FC, PropsWithChildren } from "react";

import { useContext } from "react";
import { createPortal } from "react-dom";

import { SurfaceContext } from "./SurfaceContext";

export type SurfaceHeaderProps = PropsWithChildren;

const SurfaceHeader: FC<SurfaceHeaderProps> = ({ children }) => {
  const context = useContext(SurfaceContext);
  if (!context) {
    throw new Error("Surface.Header must be used within a Surface component");
  }

  const { headerRef } = context;
  if (!headerRef) return null;

  return createPortal(children, headerRef);
};

export default SurfaceHeader;
