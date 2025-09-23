import { ListView, Page, Stack } from "@cardfolio/ui";
import Button from "@cardfolio/ui/src/components/Button/Button";
import { createFileRoute } from "@tanstack/react-router";
import { useMemo } from "react";

import {
  calculatePlayerScore,
  getPlayerWinsLossesDraws,
  useTournamentStore,
} from "@/store/tournamentStore";

export const Route = createFileRoute("/tournament/$id/scores")({
  component: ScoresPage,
});

function ScoresPage() {
  const { id } = Route.useParams();
  const navigate = Route.useNavigate();

  const tournament = useTournamentStore((state) => state.tournaments.find((t) => t.id === id));
  if (!tournament) return <div>Tournament not found</div>;

  const scores = useMemo(() =>
    tournament.players.map(
      (player) => {
        const score = calculatePlayerScore(tournament, player);
        const { wins, losses, draws } = getPlayerWinsLossesDraws(tournament, player);

        return [player, score, wins, losses, draws] as const;
      },
    ).sort(([, scoreA], [, scoreB]) => scoreB - scoreA), [tournament]);

  return (
    <Page title={`Tourney ${tournament.name}`}>
      <Page.Toolbar>
        <Button
          onClick={() => {
            navigate({ to: `/tournament/${tournament.id}/` });
          }}
        >
          Back to matches
        </Button>
      </Page.Toolbar>
      <Page.Content>
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
      </Page.Content>
    </Page>
  );
}
