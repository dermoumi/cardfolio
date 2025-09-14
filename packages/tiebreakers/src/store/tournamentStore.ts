import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ID = string;
export type Result = "A" | "B" | "draw";

const DEFAULT_WIN_POINTS = 3;
const DEFAULT_DRAW_POINTS = 1;
const DEFAULT_LOSS_POINTS = 0;

type PlayerScore = string;

export type Player = {
  id: ID;
  name: string;
};

export type Match = {
  id: ID;
  playerA: ID;
  playerB?: ID;
  result?: Result;
};

export type Round = {
  id: ID;
  number: number;
  matches: Array<Match>;
};

export type Tournament = {
  id: ID;
  name: string;
  players: Array<Player>;
  rounds: Array<Round>;
  currentRound?: number;
  status: "setup" | "in-progress" | "top-cut" | "finished";

  // Settings
  winPoints: number;
  drawPoints: number;
  lossPoints: number;
};

type Store = {
  tournaments: Array<Tournament>;

  // actions
  createTournament: (name: string) => ID;
  removeTournament: (tournamentId: ID) => void;

  viewFirstRound: (tournamentId: ID) => void;
  viewLastRound: (tournamentId: ID) => void;
  viewPrevRound: (tournamentId: ID) => void;
  viewNextRound: (tournamentId: ID) => void;
  isViewingFirstRound: (tournamentId: ID) => boolean;
  isViewingLastRound: (tournamentId: ID) => boolean;

  addPlayer: (tournamentId: ID, playerName: string) => ID;
  renamePlayer: (tournamentId: ID, playerId: ID, newName: string) => void;
  removePlayer: (tournamentId: ID, playerId: ID) => void;
  startTournament: (tournamentId: ID) => void;

  addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result | null) => void;
  nextRound: (tournamentId: ID) => void;
  topCut: (tournamentId: ID, cutTo: number) => void;
};

function uuid() {
  return crypto.randomUUID();
}

export function getPlayerWinsLossesDraws(
  tournament: Tournament,
  player: Player,
): { wins: number; losses: number; draws: number; } {
  const [wins, losses, draws] = tournament.rounds
    .flatMap((round) => round.matches)
    .reduce(
      ([w, l, d], { playerA, playerB, result }) => {
        if (playerA === player.id) {
          if (result === "A") w++;
          else if (result === "B") l++;
          else if (result === "draw") d++;
        } else if (playerB === player.id) {
          if (result === "B") w++;
          else if (result === "A") l++;
          else if (result === "draw") d++;
        }

        return [w, l, d];
      },
      [0, 0, 0],
    );

  return { wins, losses, draws };
}

function getPlayerOpponentsIds(tournament: Tournament, player: Player): Array<ID> {
  return tournament.rounds.flatMap((round) =>
    round.matches
      .filter(
        (match) => match.playerA === player.id || match.playerB === player.id,
      )
      .map((match) => (match.playerA === player.id ? match.playerB : match.playerA))
      .filter((id): id is ID => !!id)
  );
}

function getPlayerOpponents(tournament: Tournament, player: Player): Array<Player> {
  return getPlayerOpponentsIds(tournament, player)
    .map((opponentId) => tournament.players.find((p) => p.id === opponentId))
    .filter((p): p is Player => !!p);
}

function getPlayerRoundsLost(tournament: Tournament, player: Player): Array<number> {
  const roundsLost: Array<number> = [];
  tournament.rounds.forEach((round) => {
    round.matches.forEach(({ result, playerA, playerB }) => {
      if (result && (playerA === player.id || playerB === player.id)) {
        const isPlayerA = playerA === player.id;
        if ((result === "B" && isPlayerA) || (result === "A" && !isPlayerA)) {
          roundsLost.push(round.number);
        }
      }
    });
  });
  return roundsLost;
}

function getTotalPossiblePoints({ rounds, winPoints = DEFAULT_WIN_POINTS }: Tournament): number {
  return rounds.length * winPoints;
}

function getMatchPoints(tournament: Tournament, player: Player): number {
  const { wins, draws, losses } = getPlayerWinsLossesDraws(tournament, player);
  const {
    winPoints = DEFAULT_WIN_POINTS,
    drawPoints = DEFAULT_DRAW_POINTS,
    lossPoints = DEFAULT_LOSS_POINTS,
  } = tournament;
  return wins * winPoints + draws * drawPoints + losses * lossPoints;
}

function getMWP(tournament: Tournament, player: Player): number {
  const totalPossiblePoints = getTotalPossiblePoints(tournament);
  if (totalPossiblePoints === 0) return 0;

  const matchPoints = getMatchPoints(tournament, player);
  return matchPoints / totalPossiblePoints;
}

function getOppMWP(tournament: Tournament, player: Player): number {
  const opponents = getPlayerOpponents(tournament, player);
  if (opponents.length === 0) return 0;

  const totalMwp = opponents.reduce((sum, opponent) => sum + getMWP(tournament, opponent), 0);
  return totalMwp / opponents.length;
}

function getOppsOppMWP(tournament: Tournament, player: Player): number {
  const opponents = getPlayerOpponents(tournament, player);
  if (opponents.length === 0) return 0;

  const totalOowp = opponents.reduce((sum, opponent) => sum + getOppMWP(tournament, opponent), 0);
  return totalOowp / opponents.length;
}

export function calculatePlayerScore(tournament: Tournament, player: Player): PlayerScore {
  const matchPoints = getMatchPoints(tournament, player);

  const oppMWP = getOppMWP(tournament, player);
  const oppOppsMWP = getOppsOppMWP(tournament, player);
  const roundsLost = getPlayerRoundsLost(tournament, player);
  const sumSqRoundsLost = roundsLost.reduce((sum, rounds) => sum + rounds * rounds, 0);

  const AA = matchPoints.toString().padStart(2, "0");
  const BBB = Math.min(Math.round(oppMWP * 1000), 999).toString().padStart(3, "0");
  const CCC = Math.min(Math.round(oppOppsMWP * 1000), 999).toString().padStart(3, "0");
  const DDD = Math.min(sumSqRoundsLost, 999).toString().padStart(3, "0");
  return `${AA}${BBB}${CCC}${DDD}`;
}

function shuffleArray<T>(array: Array<T>): Array<T> {
  const arr = [...array];
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    [arr[i], arr[j]] = [arr[j]!, arr[i]!];
  }
  return arr;
}

// Sorts players by their score string in descending order
// pair adjancent, avoid rematches if possible by searching forward
// If odd number of players, last player gets a bye
function generateSwissPairings(tournament: Tournament): Array<Match> {
  const playersByScore = shuffleArray(tournament.players).sort((a, b) => {
    const scoreA = calculatePlayerScore(tournament, a);
    const scoreB = calculatePlayerScore(tournament, b);

    return scoreB.localeCompare(scoreA);
  });

  const matches: Array<Match> = [];
  const pairedPlayerIds = new Set<ID>();

  for (let i = 0; i < playersByScore.length; i++) {
    const playerA = playersByScore[i];
    if (!playerA) continue;
    if (pairedPlayerIds.has(playerA.id)) continue;

    const opponents = getPlayerOpponentsIds(tournament, playerA);

    let playerB: Player | undefined;

    // Try to find an opponent
    for (let j = i + 1; j < playersByScore.length; j++) {
      const potentialOpp = playersByScore[j];
      if (!potentialOpp) continue;

      if (!pairedPlayerIds.has(potentialOpp.id) && !opponents.includes(potentialOpp.id)) {
        playerB = potentialOpp;
        break;
      }
    }

    // If no opponent found without rematch, just take the next available player
    if (!playerB) {
      for (let j = i + 1; j < playersByScore.length; j++) {
        const potentialOpponent = playersByScore[j];
        if (!potentialOpponent) continue;

        if (!pairedPlayerIds.has(potentialOpponent.id)) {
          playerB = potentialOpponent;
          break;
        }
      }
    }

    if (playerB) {
      matches.push({
        id: uuid(),
        playerA: playerA.id,
        playerB: playerB.id,
      });
      pairedPlayerIds.add(playerA.id);
      pairedPlayerIds.add(playerB.id);
    } else {
      // Bye
      matches.push({
        id: uuid(),
        playerA: playerA.id,
      });
      pairedPlayerIds.add(playerA.id);
    }
  }

  return matches;
}

export const useTournamentStore = create<Store>()(
  persist(
    (set, get) => ({
      tournaments: [],
      createTournament: (name: string) => {
        const newTournament: Tournament = {
          id: uuid(),
          name,
          players: [],
          rounds: [],
          status: "setup",
          winPoints: DEFAULT_WIN_POINTS,
          drawPoints: DEFAULT_DRAW_POINTS,
          lossPoints: DEFAULT_LOSS_POINTS,
        };
        set((state) => ({
          tournaments: [...state.tournaments, newTournament],
        }));
        return newTournament.id;
      },
      removeTournament: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.filter(t => t.id !== tournamentId),
        }));
      },
      viewFirstRound: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              return {
                ...tournament,
                currentRound: tournament.rounds.length > 0 ? 0 : undefined,
              };
            }
            return tournament;
          }),
        }));
      },
      viewLastRound: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              return {
                ...tournament,
                currentRound: tournament.rounds.length > 0
                  ? tournament.rounds.length - 1
                  : undefined,
              };
            }
            return tournament;
          }),
        }));
      },
      viewPrevRound: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              const current = tournament.currentRound
                ?? (tournament.rounds.length > 0 ? tournament.rounds.length - 1 : undefined);
              if (current === undefined || current <= 0) return tournament;
              return {
                ...tournament,
                currentRound: current - 1,
              };
            }
            return tournament;
          }),
        }));
      },
      viewNextRound: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              const current = tournament.currentRound
                ?? (tournament.rounds.length > 0 ? tournament.rounds.length - 1 : undefined);
              if (current === undefined || current >= tournament.rounds.length - 1) {
                return tournament;
              }
              return {
                ...tournament,
                currentRound: current + 1,
              };
            }
            return tournament;
          }),
        }));
      },
      isViewingFirstRound: (tournamentId: ID) => {
        const tournament = get().tournaments.find(t => t.id === tournamentId);
        if (!tournament) return false;
        const current = tournament.currentRound
          ?? (tournament.rounds.length > 0 ? tournament.rounds.length - 1 : undefined);
        return current === 0;
      },
      isViewingLastRound: (tournamentId: ID) => {
        const tournament = get().tournaments.find(t => t.id === tournamentId);
        if (!tournament) return false;
        const current = tournament.currentRound
          ?? (tournament.rounds.length > 0 ? tournament.rounds.length - 1 : undefined);
        return current
          === (tournament.rounds.length > 0 ? tournament.rounds.length - 1 : undefined);
      },
      addPlayer: (tournamentId: ID, playerName: string) => {
        const newPlayer: Player = {
          id: uuid(),
          name: playerName,
        };
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              return {
                ...tournament,
                players: [...tournament.players, newPlayer],
              };
            }
            return tournament;
          }),
        }));
        return newPlayer.id;
      },
      renamePlayer: (tournamentId: ID, playerId: ID, newName: string) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              return {
                ...tournament,
                players: tournament.players.map((player) => {
                  if (player.id === playerId) {
                    return { ...player, name: newName };
                  }
                  return player;
                }),
              };
            }
            return tournament;
          }),
        }));
      },
      removePlayer: (tournamentId: ID, playerId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              return {
                ...tournament,
                players: tournament.players.filter((p) => p.id !== playerId),
              };
            }
            return tournament;
          }),
        }));
      },

      startTournament: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId && tournament.status === "setup") {
              const firstRoundMatches = generateSwissPairings(tournament);
              const firstRound: Round = {
                id: uuid(),
                number: 1,
                matches: firstRoundMatches,
              };
              return {
                ...tournament,
                rounds: [firstRound],
                currentRound: 0,
                status: "in-progress",
              };
            }
            return tournament;
          }),
        }));
      },

      addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result | null) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              const updatedRounds = tournament.rounds.map((round) => {
                if (round.id === roundId) {
                  const updatedMatches = round.matches.map((match) => {
                    if (match.id === matchId) {
                      return { ...match, result: result ?? undefined };
                    }
                    return match;
                  });
                  return { ...round, matches: updatedMatches };
                }
                return round;
              });

              return {
                ...tournament,
                rounds: updatedRounds,
              };
            }
            return tournament;
          }),
        }));
      },
      nextRound: (tournamentId: ID) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId && tournament.status === "in-progress") {
              const newRoundNumber = tournament.rounds.length + 1;
              const newRoundMatches = generateSwissPairings(tournament);
              const newRound: Round = {
                id: uuid(),
                number: newRoundNumber,
                matches: newRoundMatches,
              };
              return {
                ...tournament,
                rounds: [...tournament.rounds, newRound],
                currentRound: tournament.rounds.length,
              };
            }
            return tournament;
          }),
        }));
      },
      topCut: (tournamentId: ID, cutTo: number) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId && tournament.status === "in-progress") {
              const playersByScore = [...tournament.players].sort((a, b) => {
                const scoreA = calculatePlayerScore(tournament, a);
                const scoreB = calculatePlayerScore(tournament, b);
                if (scoreA > scoreB) return -1;
                if (scoreA < scoreB) return 1;
                return 0;
              });
              const topPlayers = playersByScore.slice(0, cutTo);
              return {
                ...tournament,
                players: topPlayers,
                rounds: [],
                status: "top-cut",
              };
            }
            return tournament;
          }),
        }));
      },
    }),
    { name: "tiebreakers-storage" },
  ),
);
