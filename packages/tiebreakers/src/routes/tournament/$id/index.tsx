import type { Round, Tournament } from "@/store/tournamentStore";
import type { FC } from "react";

import { Button, ListView, Page, Stack, Surface, TextInput } from "@cardfolio/ui";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import MatchComponent from "@/components/MatchComponent";
import { useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/tournament/$id/")({
  component: TournamentPage,
});

function TournamentPage() {
  const { id } = Route.useParams();
  const tournament = useTournamentStore((state) => state.tournaments.find((t) => t.id === id));
  if (!tournament) return <div>Tournament not found</div>;

  return (
    <Page title={`Tourney ${tournament.name}`}>
      {tournament.status === "setup" && <Setup tournament={tournament} />}
      {tournament.status === "in-progress" && <MatchesView tournament={tournament} />}
    </Page>
  );
}

type SetupProps = {
  tournament: Tournament;
};

const Setup: FC<SetupProps> = ({ tournament }) => {
  const navigate = useNavigate();
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

  const handleGoBack = useCallback(() => {
    navigate({ to: `/` });
  }, []);

  return (
    <>
      <Page.Toolbar>
        <Button onClick={handleGoBack}>Back</Button>
        <Page.ToolbarSpacer />
        <Button disabled={tournament.players.length <= 2} onClick={handleStartTournament}>
          Start tournament
        </Button>
      </Page.Toolbar>
      <Page.Content>
        <h3>Players</h3>
        <Surface>
          <form onSubmit={handleAddPlayer}>
            <Stack horizontal gap="small">
              <Stack.Stretch>
                <TextInput
                  name="new-player-name"
                  placeholder="Add a player..."
                  value={playerName}
                  onChange={setPlayerName}
                />
              </Stack.Stretch>
              <Button
                disabled={!playerName || tournament.status !== "setup"}
                type="submit"
              >
                Add
              </Button>
            </Stack>
          </form>
        </Surface>
        <Surface variant="transparent">
          <Stack gap="small">
            {tournament.players.map((player) => (
              <Stack horizontal gap="small" key={player.id}>
                <Stack.Stretch>
                  <TextInput
                    name={`player-${player.id}-name`}
                    placeholder="Player name"
                    value={player.name}
                    onChange={(name) => renamePlayer(tournament.id, player.id, name)}
                  />
                </Stack.Stretch>
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
              </Stack>
            ))}
          </Stack>
        </Surface>
      </Page.Content>
    </>
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

  const handleGoBack = useCallback(() => {
    navigate({ to: `/` });
  }, [navigate]);

  return (
    <>
      <Page.Toolbar>
        <Button onClick={handleGoBack}>Back</Button>
        <Page.ToolbarSpacer />
        <Button onClick={handleShowScores}>
          Player scores
        </Button>
        <Button
          disabled={!allMatchesCompleted || !isViewingLastRound(tournament.id)}
          onClick={handleEndRound}
        >
          End round
        </Button>
      </Page.Toolbar>
      <Page.Content>
        <Stack>
          <Stack horizontal gap="small">
            <Button
              disabled={isViewingFirstRound(tournament.id)}
              onClick={() => viewPrevRound(tournament.id)}
            >
              ←
            </Button>
            <Stack.Stretch>
              <h3>Round {round.number}</h3>
            </Stack.Stretch>
            <Button
              disabled={isViewingLastRound(tournament.id)}
              onClick={() => viewNextRound(tournament.id)}
            >
              →
            </Button>
            <Button
              disabled={isViewingLastRound(tournament.id)}
              onClick={() => viewLastRound(tournament.id)}
            >
              ⇥
            </Button>
          </Stack>
          <ListView>
            {round.matches.map((match) => (
              <ListView.Item key={match.id}>
                <MatchComponent match={match} round={round} tournament={tournament} />
              </ListView.Item>
            ))}
          </ListView>
          <div>
          </div>
        </Stack>
      </Page.Content>
    </>
  );
};
