import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ID = string;
export type Result = "A" | "B" | "draw";

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

  addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result) => void;
  nextRound: (tournamentId: ID) => void;
  topCut: (tournamentId: ID, cutTo: number) => void;
};

function uuid() {
  return crypto.randomUUID();
}

function getPlayerMatchWins(tournament: Tournament, player: Player): number {
  const matches = tournament.rounds.flatMap((round) => round.matches);

  return matches.reduce((count, { result, playerA, playerB }) => {
    if ((result === "A") && playerA === player.id) return count + 1;
    if (result === "B" && playerB === player.id) return count + 1;
    return count;
  }, 0);
}

function getPlayerMatchDraws(tournament: Tournament, player: Player): number {
  const matches = tournament.rounds.flatMap((round) => round.matches);

  return matches.reduce((count, { result, playerA, playerB }) => {
    if (result === "draw" && (playerA === player.id || playerB === player.id)) return count + 1;
    return count;
  }, 0);
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

export function calculatePlayerScore(tournament: Tournament, player: Player): PlayerScore {
  const opponents = getPlayerOpponents(tournament, player);

  const wins = getPlayerMatchWins(tournament, player);
  const draws = getPlayerMatchDraws(tournament, player);
  const matchPoints = wins * 3 + draws;

  const opponentsMatchWinPercentage = opponents.length
    ? opponents.reduce((sum, opponent) => {
      const opponentWins = getPlayerMatchWins(tournament, opponent);
      const opponentDraws = getPlayerMatchDraws(tournament, opponent);
      const opponentMatchPoints = opponentWins * 3 + opponentDraws;
      const totalPossiblePoints = tournament.rounds.length * 3;
      return sum + (opponentMatchPoints / totalPossiblePoints);
    }, 0) / opponents.length
    : 0;

  const opponentsOpponentsMatchWinPercentage = opponents.length
    ? opponents.reduce((sum, opponent) => {
      const opponentOpponents = getPlayerOpponents(tournament, opponent);
      const opponentOpponentsMwp = opponentOpponents.length
        ? opponentOpponents.reduce((oppSum, oppOpponent) => {
          const opponentWins = getPlayerMatchWins(tournament, oppOpponent);
          const opponentDraws = getPlayerMatchDraws(tournament, oppOpponent);
          const oppOpponentMatchPoints = opponentWins * 3 + opponentDraws;
          const totalPossiblePoints = tournament.rounds.length * 3;
          return oppSum + (oppOpponentMatchPoints / totalPossiblePoints);
        }, 0) / opponentOpponents.length
        : 0;
      return sum + opponentOpponentsMwp;
    }, 0) / opponents.length
    : 0;

  const roundsLost = getPlayerRoundsLost(tournament, player);
  const sumSqRoundsLost = roundsLost.reduce((sum, rounds) => sum + rounds * rounds, 0);

  // Format AABBBCCCDDD where:
  // AA = match points, zero-padded to 2 digits
  // BBB = opponents' match win percentage, capped at 999, zero-padded to 3 digits
  // CCC = opponents' opponents' match win percentage, capped at 999, zero-padded to 3 digits
  // DDD = sum of squares of rounds lost, capped at 999, zero-padded to 3 digits
  return (
    String(matchPoints).padStart(2, "0")
    + String(Math.min(Math.round(opponentsMatchWinPercentage * 1000), 999)).padStart(3, "0")
    + String(Math.min(Math.round(opponentsOpponentsMatchWinPercentage * 1000), 999)).padStart(
      3,
      "0",
    )
    + String(Math.min(sumSqRoundsLost, 999)).padStart(3, "0")
  );
}

// Sorts players by their score string in descending order
// pair adjancent, avoid rematches if possible by searching forward
// If odd number of players, last player gets a bye
function generateSwitchPairings(tournament: Tournament): Array<Match> {
  const playersByScore = [...tournament.players].sort((a, b) => {
    const scoreA = calculatePlayerScore(tournament, a);
    const scoreB = calculatePlayerScore(tournament, b);
    if (scoreA > scoreB) return -1;
    if (scoreA < scoreB) return 1;
    return 0;
  });

  const matches: Array<Match> = [];
  const pairedPlayerIds = new Set<ID>();

  for (let i = 0; i < playersByScore.length; i++) {
    const playerA = playersByScore[i];
    if (!playerA) continue;
    if (pairedPlayerIds.has(playerA.id)) continue;

    let playerB: Player | undefined;
    for (let j = i + 1; j < playersByScore.length; j++) {
      const potentialOpponent = playersByScore[j];
      if (!potentialOpponent) continue;
      const opponents = getPlayerOpponentsIds(tournament, playerA);
      if (
        !pairedPlayerIds.has(potentialOpponent.id)
        && !opponents.includes(potentialOpponent.id)
      ) {
        playerB = potentialOpponent;
        break;
      }
    }

    if (!playerB) {
      // If no opponent found without rematch, just take the next available player
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
              const firstRoundMatches = generateSwitchPairings(tournament);
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

      addResult: (tournamentId: ID, roundId: ID, matchId: ID, result: Result) => {
        set((state) => ({
          tournaments: state.tournaments.map((tournament) => {
            if (tournament.id === tournamentId) {
              const updatedRounds = tournament.rounds.map((round) => {
                if (round.id === roundId) {
                  const updatedMatches = round.matches.map((match) => {
                    if (match.id === matchId && !match.result) {
                      return { ...match, result };
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
              const newRoundMatches = generateSwitchPairings(tournament);
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
    {
      name: "tiebreakers-storage", // name of the item in the storage (must be unique)
    },
  ),
);
