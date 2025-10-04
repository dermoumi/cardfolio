import { Button, FloatingAction, ListView, Page, Stack } from "@cardfolio/ui";
import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import MatchComponent from "@/components/MatchComponent";
import { findTournament, getCurrentRound, useTournamentStore } from "@/store/tournamentStore";

export const Route = createFileRoute("/$id/")({
  component: TournamentPage,
});

function TournamentPage() {
  const { id } = Route.useParams();
  const tournament = useTournamentStore(({ tournaments }) => findTournament(tournaments, id));
  if (!tournament) return <div>Tournament not found</div>;

  const currentRound = getCurrentRound(tournament);
  const round = useMemo(() => {
    if (currentRound === undefined) throw new Error("No current round");
    const r = tournament.rounds[currentRound];
    if (!r) throw new Error("Current round not found");
    return r;
  }, [currentRound, tournament.rounds]);

  const allMatchesCompleted = useMemo(() => {
    return round.matches.every((match) => match.result);
  }, [round.matches]);

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
    navigate({ to: `/${tournament.id}/scores` });
  }, [navigate, tournament.id]);

  return (
    <Page>
      <Page.Header
        title={tournament.name}
        backAction={<Page.BackButton from={Route.fullPath} to="/" />}
        actions={<Button onClick={handleShowScores} variant="subtle">Scores</Button>}
      />
      <FloatingAction
        disabled={!allMatchesCompleted || !isViewingLastRound(tournament.id)}
        onClick={handleEndRound}
        icon="calendarClock"
      >
        End round
      </FloatingAction>
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
          {round.matches.map((match, index) => (
            <ListView.Item key={match.id}>
              <MatchComponent
                match={match}
                round={round}
                tournament={tournament}
                table={index + 1}
              />
            </ListView.Item>
          ))}
        </ListView>
        <div>
        </div>
      </Stack>
    </Page>
  );
}
