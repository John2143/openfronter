import { Lobby } from '../types';

export interface FetchLobbiesParams {
  completed?: boolean | null;
  after?: number | null;
  mapName?: string;
}

export const fetchLobbies = async (params: FetchLobbiesParams = {}): Promise<Lobby[]> => {
  const { completed, after, mapName } = params;
  
  // Build URL with query parameters
  const url = new URL('/api/v1/lobbies');
  
  if (completed !== null && completed !== undefined) {
    url.searchParams.append('completed', completed.toString());
  }
  
  if (after !== null && after !== undefined) {
    url.searchParams.append('after', after.toString());
  }
  
  if (mapName) {
    url.searchParams.append('map_name', mapName);
  }
  
  const response = await fetch(url.toString());
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  const data = await response.json();
  return Array.isArray(data) ? data : [];
};

export const markGameForAnalysis = async (gameId: string) => {
  const res = await fetch(`/api/v1/game/${gameId}/analyze`, { method: 'POST' });
  if (!res.ok) {
    throw new Error(`HTTP error! status: ${res.status}`);
  }
};

export const unmarkGameForAnalysis = async (gameId: string) => {
  const res = await fetch(`/api/v1/game/${gameId}/analyze`, { method: 'DELETE' });
  if (!res.ok) {
    throw new Error(`HTTP error! status: ${res.status}`);
  }
};
