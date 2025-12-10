import type { YgoCard, YgoCardFilters } from "@/types";
import type { InfiniteData, UseInfiniteQueryResult } from "@tanstack/react-query";

import { useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

type YgoCardResponse = {
  cards: YgoCard[];
  next: string | null;
};

export function useYgoCards(
  filters: YgoCardFilters,
  limit: number,
): UseInfiniteQueryResult<InfiniteData<YgoCardResponse, string | null>> {
  const filtersParams = useMemo(() => {
    const params = new URLSearchParams();
    params.append("limit", limit.toString());

    Object.entries(filters).forEach(([key, value]) => {
      if (Array.isArray(value)) {
        value.forEach((v) => params.append(key, v));
      } else if (value) {
        params.append(key, value.toString());
      }
    });

    return params;
  }, [filters, limit]);

  return useInfiniteQuery({
    queryKey: ["cards", filters],
    queryFn: async ({ pageParam }) => {
      let params = filtersParams;
      if (pageParam) {
        params = new URLSearchParams(filtersParams);
        params.append("cursor", pageParam);
      }

      const response = await fetch(`/api/v1/ygo/cards?${params}`);
      if (!response.ok) throw new Error("Failed to fetch cards");

      return response.json();
    },
    getNextPageParam: (lastPage) => lastPage.next,
    initialPageParam: null as string | null,
  });
}
