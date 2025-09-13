import type { Player } from "@/store/tournamentStore";

import Button from "@cardfolio/ui/src/components/Button/Button";
import { createFileRoute } from "@tanstack/react-router";
import { useMemo } from "react";

import { calculatePlayerScore, useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/tournament/$id/scores")({
  component: ScoresPage,
});

function ScoresPage() {
  const { id } = Route.useParams();
  const navigate = Route.useNavigate();

  const tournament = useTournamentStore((state) => state.tournaments.find((t) => t.id === id));
  if (!tournament) return <div>Tournament not found</div>;

  const scores = useMemo(() => {
    const unordered: Array<[Player, string]> = tournament.players.map((player) => [
      player,
      calculatePlayerScore(tournament, player),
    ]);

    return unordered.sort(([, scoreA], [, scoreB]) => {
      // Sort by score string comparison
      return scoreB.localeCompare(scoreA);
    });
  }, [tournament]);

  return (
    <div>
      <h2>Scores for {tournament.name}</h2>
      <ol>
        {scores.map(([player, score]) => {
          return (
            <li key={player.id}>
              <span style={{ fontFamily: "monospace" }}>{score}</span> {player.name}
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
