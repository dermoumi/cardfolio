import { createContext } from "react";

export type RadioGroupContextValue = {
  name: string;
  value: string | null;
  onChange: (value: string) => void;
  disabled?: boolean;
};

export const RadioGroupContext = createContext<RadioGroupContextValue | null>(null);

export default RadioGroupContext;
