// Types for lobby data
export interface Lobby {
  game_id: string;
  teams: number | null;
  max_players: number;
  game_map: string;
  approx_num_players: number;
  first_seen_unix_sec: number;
  last_seen_unix_sec: number;
  completed: boolean;
  analysis_complete: boolean;
}

export type SortBy = 'last_seen' | 'players' | 'map';

export type GameStatus = 'completed' | 'full' | 'active' | 'in-progress';
