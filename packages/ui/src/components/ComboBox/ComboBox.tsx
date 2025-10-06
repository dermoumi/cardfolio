import type { FC, PropsWithChildren } from "react";

import ComboBoxOption from "./ComboBoxOption";

export type ComboBoxProps = PropsWithChildren<{
  name: string;
  value?: string;
  onChange?: (value: string) => void;
}>;

export type ComboBoxComponent = FC<ComboBoxProps> & {
  Option: typeof ComboBoxOption;
};

const ComboBox: ComboBoxComponent = ({ name, children, value, onChange }) => {
  return (
    <select name={name} value={value} onChange={(e) => onChange?.(e.target.value)}>
      {children}
    </select>
  );
};

ComboBox.Option = ComboBoxOption;

export default ComboBox;
