import React, { useState } from 'react';
import axios from 'axios';
import Button from 'react-bootstrap/Button';
import Form from 'react-bootstrap/Form';
import './App.css';

const Signup = () => {
    const [name, setName] = useState('');
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [result, setResult] = useState('');

    const handleSignUp =  (event) => {
      event.preventDefault();

      axios.post(`/signup`, { name, email, password })
        .then(res => {
          setResult(JSON.stringify(res));
          setTimeout(() => {
            window.location.replace("/");
            //window.location.href = '/';
          }, 3000);
        }).catch( e => {
          setResult("error: " + e);
        });
    };

  return (
    <Form onSubmit={handleSignUp}>
    <Form.Group className="mb-3" controlId="formBasicText">
      <Form.Label>Name</Form.Label>
      <Form.Control value={name} onChange={(e) => setName(e.target.value)} type="text" placeholder="Enter name" />
    </Form.Group>

    <Form.Group className="mb-3" controlId="formBasicEmail">
      <Form.Label>Email address</Form.Label>
      <Form.Control value={email} onChange={(e) => setEmail(e.target.value)} type="email" placeholder="Enter email" />
    </Form.Group>

    <Form.Group className="mb-3" controlId="formBasicPassword">
      <Form.Label>Password</Form.Label>
      <Form.Control value={password} onChange={(e) => setPassword(e.target.value)} type="password" placeholder="Password" />
    </Form.Group>

    <Button variant="primary" type="submit">
      Submit
    </Button>

    <div>result: {result}</div>
  </Form>
  );
};

export default Signup;
