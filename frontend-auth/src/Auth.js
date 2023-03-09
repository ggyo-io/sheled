import React, { useState } from 'react';
import Button from 'react-bootstrap/Button';
import Stack from 'react-bootstrap/Stack';
import Tab from 'react-bootstrap/Tab';
import Tabs from 'react-bootstrap/Tabs';
import Signup from './Signup';
import Login from './Login';

const Auth = () => {
    const State = {
        Landing: "landing",
        Login: "login",
        Signup: "singup"
    }

    const [state, setState] = useState(State.Landing);

    return (
        <Tabs
            id="controlled-tab-example"
            activeKey={state}
            onSelect={(k) => setState(k)}
            className="mb-3"
        >
            <Tab eventKey={State.Landing} title="Auth">
                <h1 class="display-4">Welcome To GGYO Chess</h1>
                <p class="lead">Log in with your account to continue.</p>
                <hr class="my-4" />
                <Stack direction="horizontal" gap={2}>
                    <Button onClick={() => setState(State.Login)}>Login</Button>
                    <Button onClick={() => setState(State.Signup)}>Sign Up</Button>
                </Stack>
            </Tab>
            <Tab eventKey={State.Login} title="Login">
                <Login />
            </Tab>
            <Tab eventKey={State.Signup} title="Sign Up">
                <Signup />
            </Tab>
        </Tabs>

    );
};

export default Auth;
