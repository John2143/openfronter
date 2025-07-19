import React from 'react';
import { useParams, useNavigate } from 'react-router-dom';

const GameDetail: React.FC = () => {
  const { gameID } = useParams();
  const navigate = useNavigate();

  const handleBack = () => {
    navigate(-1);
  };

  return (
    <div className="game-detail">
      {/* Back button */}
      <button onClick={handleBack} className="back-button">
        ← Back
      </button>

      {/* Game ID heading */}
      <h1>Game {gameID}</h1>

      {/* Placeholder sections for stats */}
      <div className="stats-sections">
        <section className="stats-section">
          <h2>Game Statistics</h2>
          <p>Statistics for game {gameID} will be displayed here.</p>
        </section>

        <section className="stats-section">
          <h2>Player Information</h2>
          <p>Player details and information will be shown here.</p>
        </section>

        <section className="stats-section">
          <h2>Game Progress</h2>
          <p>Current game progress and status will be displayed here.</p>
        </section>

        <section className="stats-section">
          <h2>Recent Activity</h2>
          <p>Recent game activity and updates will be shown here.</p>
        </section>
      </div>
    </div>
  );
};

export default GameDetail;
