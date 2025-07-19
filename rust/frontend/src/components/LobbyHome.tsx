import React from 'react';
import { useLobbies } from '../hooks/useLobbies';
import { FilterControls, SortControls, LobbiesTable, LoadingSpinner, ErrorMessage } from './';

function LobbyHome() {
  const {
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
    refreshLobbies
  } = useLobbies();

  return (
    <div className="App">
      <header className="App-header">
        <h1>🎮 OpenFronter Lobbies</h1>
        <p>Discover and join game lobbies</p>
        <div style={{ marginTop: '1rem' }}>
          <button 
            onClick={refreshLobbies}
            disabled={loading}
            style={{
              padding: '10px 20px',
              backgroundColor: loading ? '#6c757d' : '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: loading ? 'not-allowed' : 'pointer',
              fontSize: '1rem'
            }}
          >
            {loading ? 'Refreshing...' : '🔄 Refresh Lobbies'}
          </button>
        </div>
      </header>
      
      <main className="App-main">
        <section className="lobbies-section">
          <h2>Available Lobbies</h2>
          
          <FilterControls
            completedFilter={completedFilter}
            setCompletedFilter={setCompletedFilter}
            afterFilter={afterFilter}
            setAfterFilter={setAfterFilter}
            mapFilter={mapFilter}
            setMapFilter={setMapFilter}
            lobbies={lobbies}
          />
          
          {!loading && !error && lobbies.length > 0 && (
            <SortControls
              showActiveOnly={showActiveOnly}
              setShowActiveOnly={setShowActiveOnly}
              sortBy={sortBy}
              setSortBy={setSortBy}
            />
          )}
          
          {loading && <LoadingSpinner />}
          {error && <ErrorMessage message={error} />}
          {!loading && !error && <LobbiesTable lobbies={getFilteredAndSortedLobbies()} />}
          
          {!loading && !error && lobbies.length > 0 && (
            <div style={{ textAlign: 'center', marginTop: '2rem' }}>
              <p style={{ color: '#666' }}>
                Showing {getFilteredAndSortedLobbies().length} of {lobbies.length} {lobbies.length === 1 ? 'lobby' : 'lobbies'}
                {showActiveOnly && ' (active games only)'}
              </p>
            </div>
          )}
        </section>
      </main>
      
      <footer className="App-footer">
        <p>&copy; 2024 OpenFronter. Connect and play together!</p>
      </footer>
    </div>
  );
}

export default LobbyHome;
