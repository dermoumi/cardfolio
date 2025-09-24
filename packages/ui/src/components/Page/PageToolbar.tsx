import type { FC, PropsWithChildren } from "react";

import { useContext } from "react";
import { createPortal } from "react-dom";

import { HeaderContext } from "./HeaderContext";

export type PageToolbarProps = PropsWithChildren;

const PageToolbar: FC<PageToolbarProps> = ({ children }) => {
  const headerContext = useContext(HeaderContext);
  if (!headerContext) {
    throw new Error("Page.Toolbar must be used within a Page component");
  }

  const { toolbarRef } = headerContext;
  if (!toolbarRef) return null;

  return createPortal(children, toolbarRef);
};

export default PageToolbar;
