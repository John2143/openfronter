import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';

function GameDetail() {
  const { gameID } = useParams<{ gameID: string }>();
  const navigate = useNavigate();

  const handleBackToLobbies = () => {
    navigate('/');
  };

  return (
    <div className="App">
      <header className="App-header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '20px' }}>
          <button 
            onClick={handleBackToLobbies}
            style={{
              padding: '10px 20px',
              backgroundColor: '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '5px',
              cursor: 'pointer',
              fontSize: '14px',
              transition: 'background-color 0.2s ease'
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = '#0056b3';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = '#007bff';
            }}
          >
            ← Back to Lobbies
          </button>
          <div>
            <h1>🎮 Game Detail</h1>
            <p>Game ID: {gameID}</p>
          </div>
        </div>
      </header>
      
      <main className="App-main">
        <section>
          <h2>Game Details</h2>
          <p>This is a placeholder for the game detail page.</p>
          <p>Game ID from URL: <strong>{gameID}</strong></p>
        </section>
      </main>
      
      <footer className="App-footer">
        <p>&copy; 2024 OpenFronter. Connect and play together!</p>
      </footer>
    </div>
  );
}

export default GameDetail;
