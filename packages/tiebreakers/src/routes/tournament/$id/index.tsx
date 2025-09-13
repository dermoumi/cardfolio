import type { Match, Round, Tournament } from "@/store/tournamentStore";
import type { FC } from "react";

import { Button, TextInput } from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/tournament/$id/")({
  component: TournamentPage,
});

function TournamentPage() {
  const { id } = Route.useParams();
  const tournament = useTournamentStore((state) => state.tournaments.find((t) => t.id === id));
  if (!tournament) return <div>Tournament not found</div>;

  return (
    <div>
      <h2>
        Tourney {tournament.name}
      </h2>
      {tournament.status === "setup" && <Setup tournament={tournament} />}
      {tournament.status === "in-progress" && <MatchesView tournament={tournament} />}
    </div>
  );
}

type SetupProps = {
  tournament: Tournament;
};

const Setup: FC<SetupProps> = ({ tournament }) => {
  const addPlayer = useTournamentStore((state) => state.addPlayer);
  const removePlayer = useTournamentStore((state) => state.removePlayer);
  const renamePlayer = useTournamentStore((state) => state.renamePlayer);

  const startTournament = useTournamentStore((state) => state.startTournament);

  const [playerName, setPlayerName] = useState("");

  const handleAddPlayer = () => {
    if (!playerName || tournament.status !== "setup") return;

    addPlayer(tournament.id, playerName);
    setPlayerName("");
  };

  const handleStartTournament = () => {
    startTournament(tournament.id);
  };

  return (
    <div>
      <h3>Player list</h3>
      <ul>
        {tournament.players.map((player) => (
          <li key={player.id}>
            <TextInput
              placeholder="Player name"
              value={player.name}
              onChange={(name) => renamePlayer(tournament.id, player.id, name)}
            />
            <Button
              disabled={tournament.status !== "setup"}
              onClick={() => {
                if (
                  tournament.status !== "setup"
                  || !window.confirm(`Remove player "${player.name}"?`)
                ) return;

                removePlayer(tournament.id, player.id);
              }}
            >
              Remove
            </Button>
          </li>
        ))}
        <li>
          <TextInput placeholder="Player name" value={playerName} onChange={setPlayerName} />
          <Button disabled={!playerName || tournament.status !== "setup"} onClick={handleAddPlayer}>
            Add
          </Button>
        </li>
      </ul>
      <Button disabled={tournament.players.length <= 2} onClick={handleStartTournament}>
        Start tournament
      </Button>
    </div>
  );
};

type MatchesViewProps = {
  tournament: Tournament;
};

const MatchesView: FC<MatchesViewProps> = ({ tournament }) => {
  const currentRound = tournament.currentRound ?? (tournament.rounds.length - 1);
  const round = useMemo(() => tournament.rounds[currentRound], [currentRound, tournament.rounds]);

  return round
    ? <RoundComponent round={round} tournament={tournament} />
    : <div>No rounds yet</div>;
};

type RoundComponentProps = {
  tournament: Tournament;
  round: Round;
};

const RoundComponent: FC<RoundComponentProps> = (
  {
    round,
    tournament,
  },
) => {
  const allMatchesCompleted = useMemo(() => round.matches.every((match) => match.result), [
    round.matches,
  ]);

  const nextRound = useTournamentStore((state) => state.nextRound);
  const isViewingFirstRound = useTournamentStore((state) => state.isViewingFirstRound);
  const isViewingLastRound = useTournamentStore((state) => state.isViewingLastRound);
  const viewNextRound = useTournamentStore((state) => state.viewNextRound);
  const viewPrevRound = useTournamentStore((state) => state.viewPrevRound);
  const viewLastRound = useTournamentStore((state) => state.viewLastRound);

  const handleEndRound = useCallback(() => {
    if (!allMatchesCompleted || !isViewingLastRound(tournament.id)) return;

    nextRound(tournament.id);
  }, [allMatchesCompleted, isViewingLastRound, nextRound, tournament.id]);

  const navigate = Route.useNavigate();

  const handleShowScores = useCallback(() => {
    navigate({ to: `/tournament/${tournament.id}/scores` });
  }, [navigate, tournament.id]);

  return (
    <div>
      <h3>Round {round.number}</h3>
      <div>
        <Button
          disabled={isViewingFirstRound(tournament.id)}
          onClick={() => viewPrevRound(tournament.id)}
        >
          Previous Round
        </Button>
        <Button
          disabled={isViewingLastRound(tournament.id)}
          onClick={() => viewNextRound(tournament.id)}
        >
          Next Round
        </Button>
        <Button
          disabled={isViewingLastRound(tournament.id)}
          onClick={() => viewLastRound(tournament.id)}
        >
          Last Round
        </Button>
      </div>
      <ol>
        {round.matches.map((match) => (
          <li key={match.id}>
            <MatchComponent match={match} round={round} tournament={tournament} />
          </li>
        ))}
      </ol>
      <div>
        <Button onClick={handleShowScores}>
          Player scores
        </Button>
        <Button
          disabled={!allMatchesCompleted || !isViewingLastRound(tournament.id)}
          onClick={handleEndRound}
        >
          End Round
        </Button>
      </div>
    </div>
  );
};

type MatchComponentProps = {
  tournament: Tournament;
  round: Round;
  match: Match;
};

const MatchComponent: FC<MatchComponentProps> = ({ match, round, tournament }) => {
  const addResult = useTournamentStore((state) => state.addResult);

  const playerA = useMemo(() => tournament.players.find((p) => p.id === match.playerA), [
    match.playerA,
    tournament.players,
  ]);

  const playerB = useMemo(() => tournament.players.find((p) => p.id === match.playerB), [
    match.playerB,
    tournament.players,
  ]);

  const playerAStatus = useMemo(() => {
    if (match.result === "A") return "winner";
    if (match.result === "B") return "loser";
    if (match.result === "draw") return "draw";
    return "Pending";
  }, [match.result]);

  const playerBStatus = useMemo(() => {
    if (match.result === "B") return "winner";
    if (match.result === "A") return "loser";
    if (match.result === "draw") return "draw";
    return "Pending";
  }, [match.result]);

  const resultMessage = useMemo(() => {
    if (!match.result) return "Match Pending";
    if (match.result === "draw") return "Draw";
    const winner = match.result === "A" ? playerA?.name : playerB?.name;
    return `Winner: ${winner}`;
  }, [match.result, playerA?.name, playerB?.name]);

  return (
    <div>
      <div>
        <div className={playerAStatus}>{playerA?.name}</div>
        <div className={playerBStatus}>{playerB?.name || "???"}</div>
      </div>
      <div>
        <Button onClick={() => addResult(tournament.id, round.id, match.id, "A")}>
          {playerA?.name} wins
        </Button>
        <Button
          disabled={!playerB}
          onClick={() => addResult(tournament.id, round.id, match.id, "B")}
        >
          {playerB?.name || "???"} wins
        </Button>
        <Button
          disabled={!playerB}
          onClick={() => addResult(tournament.id, round.id, match.id, "draw")}
        >
          Draw
        </Button>
        <div>{resultMessage}</div>
      </div>
    </div>
  );
};
