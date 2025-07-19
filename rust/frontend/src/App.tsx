import React from 'react';
import { Routes, Route } from 'react-router-dom';
import './App.css';
import { LobbyHome, GameDetail } from './components';

function App() {
  return (
    <Routes>
      <Route path="/" element={<LobbyHome />} />
      <Route path="/game/:gameID" element={<GameDetail />} />
    </Routes>
  );
}

export default App;
