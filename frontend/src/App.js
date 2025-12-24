import React, { useState, useEffect } from 'react';
import Container from 'react-bootstrap/Container';
import Row from 'react-bootstrap/Row';
import Col from 'react-bootstrap/Col';
import axios from 'axios';
import Dashboard from './components/Dashboard';
import './App.css';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:5000/api';

function App() {
  return (
    <div className="App">
      <Container fluid className="dashboard-container">
        <h1 className="text-center mb-4">Data Visualization Dashboard</h1>
        <Dashboard apiUrl={API_BASE_URL} />
      </Container>
    </div>
  );
}

export default App;

