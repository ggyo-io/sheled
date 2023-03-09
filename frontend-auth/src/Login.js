import React, { useState } from 'react';
import axios from 'axios';
import Button from 'react-bootstrap/Button';
import Form from 'react-bootstrap/Form';

const Login = () => {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [result, setResult] = useState('');

    const handleLogin =  (event) => {
      event.preventDefault();

      axios.post(`/login`, { email, password })
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
      <Form onSubmit={handleLogin}>
      <Form.Group className="mb-3" controlId="formBasicEmail">
        <Form.Label>Email address</Form.Label>
        <Form.Control value={email} onChange={(e) => setEmail(e.target.value)} type="email" placeholder="Enter email" />
      </Form.Group>

      <Form.Group className="mb-3" controlId="formBasicPassword">
        <Form.Label>Password</Form.Label>
        <Form.Control value={password} onChange={(e) => setPassword(e.target.value)} type="password" placeholder="Password" />
      </Form.Group>

      <Button variant="primary" type="submit" >
        Submit
      </Button>
      <div>result: {result}</div>
    </Form>
    );
  };

  export default Login;
