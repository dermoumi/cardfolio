import { ListView, Page, Stack } from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { useMemo } from "react";

import {
  calculatePlayerScore,
  getPlayerWinsLossesDraws,
  useTournamentStore,
} from "@/store/tournamentStore";

export const Route = createFileRoute("/$id/scores")({
  component: ScoresPage,
});

function ScoresPage() {
  const { id } = Route.useParams();

  const tournament = useTournamentStore((state) => state.tournaments.find((t) => t.id === id));
  if (!tournament) return <div>Tournament not found</div>;

  const scores = useMemo(() => {
    const { players, rounds, config } = tournament;
    return players.map(
      (player) => {
        const score = calculatePlayerScore(player, rounds, players, config);
        const { wins, losses, draws } = getPlayerWinsLossesDraws(player, rounds);

        return [player, score, wins, losses, draws] as const;
      },
    ).sort(([, scoreA], [, scoreB]) => scoreB - scoreA);
  }, [tournament]);

  return (
    <Page>
      <Page.Header
        title={tournament.name}
        navSlot={<Page.BackButton from={Route.fullPath} to={`/${id}`} />}
      />
      <h3>Player scores</h3>
      <ListView>
        {scores.map(([player, score, wins, losses, draws]) => {
          return (
            <ListView.Item key={player.id}>
              <Stack horizontal>
                <code>{score.toString().padStart(11, "0")}</code>
                <Stack.Stretch>
                  {player.name} ({wins}-{losses}-{draws})
                </Stack.Stretch>
              </Stack>
            </ListView.Item>
          );
        })}
      </ListView>
    </Page>
  );
}
