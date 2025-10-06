import type { PropsWithChildren } from "react";

export type ComboBoxOption = PropsWithChildren<{
  value: string;
  selected?: boolean;
}>;

const ComboBoxOption = ({ value, children, selected }: ComboBoxOption) => {
  return <option value={value} selected={selected}>{children}</option>;
};

export default ComboBoxOption;
