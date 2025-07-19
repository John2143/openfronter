import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Lobby } from '../types';
import { getGameStatus, getTimeAgo } from '../utils';
import { markGameForAnalysis, unmarkGameForAnalysis } from '../services/api';

interface LobbiesTableProps {
  lobbies: Lobby[];
}

const LobbiesTable: React.FC<LobbiesTableProps> = ({ lobbies }) => {
  const navigate = useNavigate();

  if (lobbies.length === 0) {
    return (
      <div style={{ textAlign: 'center', padding: '40px', color: '#666' }}>
        <p>No lobbies found.</p>
      </div>
    );
  }

  return (
    <div style={{ overflowX: 'auto' }}>
      <table style={{
        width: '100%',
        borderCollapse: 'collapse',
        margin: '20px 0',
        backgroundColor: 'white',
        borderRadius: '8px',
        overflow: 'hidden',
        boxShadow: '0 2px 10px rgba(0,0,0,0.1)'
      }}>
        <thead>
          <tr style={{ backgroundColor: '#f8f9fa' }}>
            <th style={{ width: '80px', textAlign: 'center', borderBottom: '2px solid #dee2e6', padding: '12px' }}>Analyze</th>
            <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Game ID</th>
            <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Map</th>
            <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Players</th>
            <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Teams</th>
            <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Status</th>
            <th style={{ padding: '12px', textAlign: 'left', borderBottom: '2px solid #dee2e6' }}>Last Seen</th>
          </tr>
        </thead>
        <tbody>
          {lobbies.map((lobby, index) => {
            const status = getGameStatus(lobby, index);
            const [isAnalyzed, setIsAnalyzed] = React.useState(lobby.analysis_complete);
            const [loading, setLoading] = React.useState(false);
            
            const handleCheckboxChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
              const checked = event.target.checked;
              setLoading(true);
              try {
                if (checked) {
                  await markGameForAnalysis(lobby.game_id);
                } else {
                  await unmarkGameForAnalysis(lobby.game_id);
                }
                setIsAnalyzed(checked);
              } catch (err: any) {
                alert(err.message);
                setIsAnalyzed(!checked);
              } finally {
                setLoading(false);
              }
            };
            
            return (
              <tr 
                key={lobby.game_id} 
                role="button"
                tabIndex={0}
                onClick={() => navigate(`/game/${lobby.game_id}`)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    navigate(`/game/${lobby.game_id}`);
                  }
                }}
                style={{
                  backgroundColor: index % 2 === 0 ? 'white' : '#f8f9fa',
                  cursor: 'pointer'
                }}
              >
                <td style={{ textAlign: 'center', borderBottom: '1px solid #dee2e6', padding: '12px' }}>
                  <input 
                    type="checkbox" 
                    checked={isAnalyzed} 
                    disabled={loading} 
                    onChange={handleCheckboxChange}
                    onClick={(e) => e.stopPropagation()}
                    style={{ cursor: 'pointer' }}
                  />
                </td>
                <td style={{ padding: '12px', borderBottom: '1px solid #dee2e6' }}>
                  <strong style={{ fontFamily: 'monospace' }}>{lobby.game_id}</strong>
                </td>
                <td style={{ padding: '12px', borderBottom: '1px solid #dee2e6' }}>
                  {lobby.game_map}
                </td>
                <td style={{ padding: '12px', borderBottom: '1px solid #dee2e6' }}>
                  <span style={{ fontWeight: 'bold' }}>
                    {lobby.approx_num_players}/{lobby.max_players}
                  </span>
                </td>
                <td style={{ padding: '12px', borderBottom: '1px solid #dee2e6' }}>
                  {lobby.teams ? lobby.teams : 'FFA'}
                </td>
                <td style={{ padding: '12px', borderBottom: '1px solid #dee2e6' }}>
                  <span style={{
                    padding: '4px 8px',
                    borderRadius: '4px',
                    fontSize: '0.8em',
                    fontWeight: 'bold',
                    backgroundColor: 
                      status === 'active' ? '#d4edda' :
                      status === 'in-progress' ? '#fff3cd' :
                      status === 'full' ? '#ffeaa7' :
                      '#f8d7da',
                    color: 
                      status === 'active' ? '#155724' :
                      status === 'in-progress' ? '#856404' :
                      status === 'full' ? '#b8860b' :
                      '#721c24'
                  }}>
                    {status.toUpperCase()}
                  </span>
                </td>
                <td style={{ padding: '12px', borderBottom: '1px solid #dee2e6', fontSize: '0.9em' }}>
                  {getTimeAgo(lobby.last_seen_unix_sec)}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};

export default LobbiesTable;
