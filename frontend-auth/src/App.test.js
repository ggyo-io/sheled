import { render, screen } from '@testing-library/react';
import App from './App';

test('renders login message', () => {
  render(<App />);
  const linkElement = screen.getByText(/Log in with your account to continue./i);
  expect(linkElement).toBeInTheDocument();
});
