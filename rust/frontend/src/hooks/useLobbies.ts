import { useState, useEffect } from 'react';
import { Lobby, SortBy } from '../types';
import { fetchLobbies } from '../services/api';

export const useLobbies = () => {
  const [lobbies, setLobbies] = useState<Lobby[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [completedFilter, setCompletedFilter] = useState<boolean | null>(null);
  const [afterFilter, setAfterFilter] = useState<number | null>(null);
  const [mapFilter, setMapFilter] = useState<string>('');
  const [showActiveOnly, setShowActiveOnly] = useState(false);
  const [sortBy, setSortBy] = useState<SortBy>('last_seen');

  const loadLobbies = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const data = await fetchLobbies({
        completed: completedFilter,
        after: afterFilter,
        mapName: mapFilter
      });
      
      setLobbies(data);
    } catch (err) {
      console.error('Error fetching lobbies:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch lobbies');
    } finally {
      setLoading(false);
    }
  };

  // Filter and sort lobbies
  const getFilteredAndSortedLobbies = () => {
    let filtered = lobbies;
    
    // Filter by active only if selected
    if (showActiveOnly) {
      filtered = lobbies.filter(lobby => !lobby.completed);
    }
    
    // Sort lobbies
    const sorted = [...filtered].sort((a, b) => {
      switch (sortBy) {
        case 'last_seen':
          return b.last_seen_unix_sec - a.last_seen_unix_sec;
        case 'players':
          return b.approx_num_players - a.approx_num_players;
        case 'map':
          return a.game_map.localeCompare(b.game_map);
        default:
          return 0;
      }
    });
    
    return sorted;
  };

  // Fetch lobbies on component mount and when filters change
  useEffect(() => {
    loadLobbies();
  }, [completedFilter, afterFilter, mapFilter]);

  return {
    lobbies,
    loading,
    error,
    completedFilter,
    setCompletedFilter,
    afterFilter,
    setAfterFilter,
    mapFilter,
    setMapFilter,
    showActiveOnly,
    setShowActiveOnly,
    sortBy,
    setSortBy,
    getFilteredAndSortedLobbies,
    refreshLobbies: loadLobbies
  };
};
