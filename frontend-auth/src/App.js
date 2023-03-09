import React, { useState } from 'react';
import Container from 'react-bootstrap/Container';
import Auth from './Auth';
import './App.css';

function App() {
  return (
  <Container className="p-3">
      <Auth/>
  </Container>
  );
}

export default App;
