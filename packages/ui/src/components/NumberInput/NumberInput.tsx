import type { FC } from "react";

import styles from "./NumberInput.module.css";

export type NumberInputProps = {
  name: string;
  value?: number;
  onChange?: (value: number) => void;
  placeholder?: string;
  required?: boolean;
  min?: number;
  max?: number;
  form?: string;
};

const NumberInput: FC<NumberInputProps> = (
  { value, name, onChange, placeholder, required, min, max, form },
) => {
  return (
    <input
      className={styles.numberInput}
      name={name}
      type="number"
      value={value}
      onChange={(e) => onChange?.(Number(e.target.value))}
      placeholder={placeholder}
      required={required}
      min={min}
      max={max}
      form={form}
    />
  );
};

export default NumberInput;
