import React from 'react';
import { Lobby } from '../types';

interface FilterControlsProps {
  completedFilter: boolean | null;
  setCompletedFilter: (filter: boolean | null) => void;
  afterFilter: number | null;
  setAfterFilter: (filter: number | null) => void;
  mapFilter: string;
  setMapFilter: (filter: string) => void;
  lobbies: Lobby[];
}

const FilterControls: React.FC<FilterControlsProps> = ({
  completedFilter,
  setCompletedFilter,
  afterFilter,
  setAfterFilter,
  mapFilter,
  setMapFilter,
  lobbies
}) => {
  const clearAllFilters = () => {
    setCompletedFilter(null);
    setAfterFilter(null);
    setMapFilter('');
  };

  return (
    <div style={{ 
      display: 'flex', 
      flexWrap: 'wrap', 
      gap: '12px', 
      marginBottom: '20px', 
      alignItems: 'center',
      justifyContent: 'center',
      padding: '16px',
      backgroundColor: '#f8f9fa',
      borderRadius: '8px'
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <label style={{ fontWeight: 'bold', color: '#495057' }}>Completed:</label>
        <button 
          onClick={() => setCompletedFilter(completedFilter === true ? null : true)}
          style={{ 
            padding: '6px 12px', 
            backgroundColor: completedFilter === true ? '#28a745' : '#6c757d', 
            color: 'white', 
            borderRadius: '4px', 
            border: 'none',
            fontSize: '0.9em',
            cursor: 'pointer'
          }}
        >
          {completedFilter === true ? '✓ Completed' : 'Show Completed'}
        </button>
        <button 
          onClick={() => setCompletedFilter(completedFilter === false ? null : false)}
          style={{ 
            padding: '6px 12px', 
            backgroundColor: completedFilter === false ? '#28a745' : '#6c757d', 
            color: 'white', 
            borderRadius: '4px', 
            border: 'none',
            fontSize: '0.9em',
            cursor: 'pointer'
          }}
        >
          {completedFilter === false ? '✓ Active' : 'Show Active'}
        </button>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <label style={{ fontWeight: 'bold', color: '#495057' }}>Time:</label>
        <button 
          onClick={() => setAfterFilter(afterFilter ? null : Date.now())}
          style={{ 
            padding: '6px 12px', 
            backgroundColor: afterFilter ? '#28a745' : '#007bff', 
            color: 'white', 
            borderRadius: '4px', 
            border: 'none',
            fontSize: '0.9em',
            cursor: 'pointer'
          }}
        >
          {afterFilter ? '✓ Recent Only' : 'Filter Recent'}
        </button>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <label style={{ fontWeight: 'bold', color: '#495057' }}>Map:</label>
        <select 
          value={mapFilter} 
          onChange={(e) => setMapFilter(e.target.value)}
          style={{ 
            padding: '6px 12px', 
            borderRadius: '4px', 
            border: '1px solid #ced4da',
            fontSize: '0.9em'
          }}
        >
          <option value=''>All Maps</option>
          {Array.from(new Set(lobbies.map(lobby => lobby.game_map))).sort().map(map => (
            <option key={map} value={map}>{map}</option>
          ))}
        </select>
      </div>

      <button 
        onClick={clearAllFilters}
        style={{ 
          padding: '6px 12px', 
          backgroundColor: '#dc3545', 
          color: 'white', 
          borderRadius: '4px', 
          border: 'none',
          fontSize: '0.9em',
          cursor: 'pointer'
        }}
      >
        🗑️ Clear Filters
      </button>
    </div>
  );
};

export default FilterControls;
