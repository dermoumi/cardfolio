import type { YgoCard } from "@/types";
import type {
  InfiniteData,
  UseInfiniteQueryResult,
} from "node_modules/@tanstack/react-query/build/legacy";

import { useInfiniteQuery } from "@tanstack/react-query";

type YgoCardResponse = {
  cards: Array<YgoCard>;
  next: string | null;
};

type YgoCardFilters = Record<string, string>;

export function useYgoCards(
  filters: YgoCardFilters,
  limit: number,
): UseInfiniteQueryResult<InfiniteData<YgoCardResponse, string | null>> {
  return useInfiniteQuery({
    queryKey: ["cards", filters],
    queryFn: async ({ pageParam }) => {
      const params = new URLSearchParams({ ...filters });
      if (pageParam) params.append("cursor", pageParam);
      params.append("limit", limit.toString());

      const response = await fetch(`/api/v1/ygo/cards?${params}`);
      if (!response.ok) throw new Error("Failed to fetch cards");

      return response.json();
    },
    getNextPageParam: (lastPage) => lastPage.next,
    initialPageParam: null as string | null,
  });
}
