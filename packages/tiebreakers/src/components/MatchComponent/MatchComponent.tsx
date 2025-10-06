import type { Match, Round, Tournament } from "@/store/tournamentStore";
import type { FC } from "react";

import { Button, Stack, Surface } from "@cardfolio/ui";
import classNames from "classnames";
import { useMemo } from "react";

import styles from "./MatchComponent.module.css";

import { useTournamentStore } from "@/store/tournamentStore";

export type MatchComponentProps = {
  tournament: Tournament;
  round: Round;
  match: Match;
  table: number;
};

const MatchComponent: FC<MatchComponentProps> = ({ match, round, tournament, table }) => {
  const addResult = useTournamentStore((state) => state.addResult);
  const { result } = match;

  const playerA = useMemo(() => tournament.players.find((p) => p.id === match.playerA), [
    match.playerA,
    tournament.players,
  ]);

  const playerB = useMemo(() => tournament.players.find((p) => p.id === match.playerB), [
    match.playerB,
    tournament.players,
  ]);

  const playerAStatus = useMemo(() => {
    if (result === "A") return styles.winner;
    if (result === "B" || result === "loss") return styles.loser;
    if (result === "draw") return styles.draw;
    return "Pending";
  }, [result]);

  const playerBStatus = useMemo(() => {
    if (result === "B") return styles.winner;
    if (result === "A" || result === "loss") return styles.loser;
    if (result === "draw") return styles.draw;
    return "Pending";
  }, [result]);

  return (
    <Surface variant="outlined" header={`Table ${table}`}>
      <Stack>
        <div className={classNames(styles.player, playerAStatus)}>
          <span>{playerA?.name}</span>
        </div>
        <div className={classNames(styles.player, playerBStatus)}>
          <span>{playerB?.name || "???"}</span>
        </div>
        <Stack horizontal gap="small">
          <Stack.Stretch>
            <Button onClick={() => addResult(tournament.id, round.id, match.id, "A")}>
              A
            </Button>
          </Stack.Stretch>
          <Stack.Stretch>
            <Button
              disabled={!playerB}
              onClick={() => addResult(tournament.id, round.id, match.id, "B")}
            >
              B
            </Button>
          </Stack.Stretch>
          <Button
            disabled={!playerB}
            onClick={() => addResult(tournament.id, round.id, match.id, "draw")}
          >
            Draw
          </Button>
          <Button
            disabled={!playerB}
            onClick={() => addResult(tournament.id, round.id, match.id, "loss")}
          >
            Loss
          </Button>
          <Button
            disabled={!match.result}
            onClick={() => addResult(tournament.id, round.id, match.id, null)}
          >
            Clear
          </Button>
        </Stack>
      </Stack>
    </Surface>
  );
};

export default MatchComponent;
