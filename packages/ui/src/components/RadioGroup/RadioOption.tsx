import type { ChangeEventHandler, FC } from "react";

import { useCallback, useContext } from "react";

import RadioGroupContext from "./context";

export type RadioOptionProps = {
  label: string;
  value: string;
  disabled?: boolean;
};

const RadioOption: FC<RadioOptionProps> = ({ label, value, disabled }) => {
  const context = useContext(RadioGroupContext);
  if (!context) {
    throw new Error("Radio.Option must be used within a RadioGroup");
  }

  const handleOnChange: ChangeEventHandler<HTMLInputElement> = useCallback((e) => {
    context.onChange(e.target.value);
  }, [context]);

  return (
    <label>
      <input
        type="radio"
        name={context.name}
        value={value}
        disabled={disabled}
        aria-disabled={disabled}
        checked={context.value === value}
        onChange={handleOnChange}
      />
      {label}
    </label>
  );
};

export default RadioOption;
