import type { FC } from "react";

export type CheckboxProps = {
  name: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
};

const Checkbox: FC<CheckboxProps> = ({ name, checked, onChange, disabled }) => {
  return (
    <input
      type="checkbox"
      name={name}
      checked={checked}
      onChange={(e) => onChange(e.target.checked)}
      disabled={disabled}
    />
  );
};

export default Checkbox;
