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
    ).sort(([, scoreA], [, scoreB]) => scoreB.localeCompare(scoreA)), [tournament]);

  return (
    <div>
      <h2>Scores for {tournament.name}</h2>
      <ol>
        {scores.map(([player, score, wins, losses, draws]) => {
          return (
            <li key={player.id}>
              <span style={{ fontFamily: "monospace" }}>{score}</span>
              <span>{player.name}</span>
              <span>
                ({wins}-{losses}-{draws})
              </span>
            </li>
          );
        })}
      </ol>
      <Button
        onClick={() => {
          navigate({ to: `/tournament/${tournament.id}/` });
        }}
      >
        Back to matches
      </Button>
    </div>
  );
}
