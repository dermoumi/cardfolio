import type { YgoCardFilters } from "@/types";
import type { FC } from "react";

import { useCallback } from "react";

import TextInput from "../TextInput";
import styles from "./YgoCardListFilter.module.css";

export type YgoCardListFilterProps = {
  filters: YgoCardFilters;
  setFilters: (filter: YgoCardFilters) => void;
};

const YgoCardListFilter: FC<YgoCardListFilterProps> = ({ filters, setFilters }) => {
  const setNameFilter = useCallback((name: string) => {
    setFilters({ ...filters, name });
  }, [setFilters, filters]);

  return (
    <div className={styles.filterBar}>
      <TextInput
        type="search"
        placeholder="Search cards..."
        value={filters.name || ""}
        onChange={setNameFilter}
      />
    </div>
  );
};

export default YgoCardListFilter;
