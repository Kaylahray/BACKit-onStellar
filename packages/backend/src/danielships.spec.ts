import { validateEnv } from './config/env.validation';

describe('danielships Backend Features (#455, #454, #453)', () => {
  it('validateEnv returns validated configuration', () => {
    const config = { PORT: '3001', NODE_ENV: 'development' };
    const validated = validateEnv(config);
    expect(validated.PORT).toBe(3001);
    expect(validated.NODE_ENV).toBe('development');
  });

  it('http-security module exports configureHttpSecurity', () => {
    const { configureHttpSecurity } = require('./security/http-security');
    expect(typeof configureHttpSecurity).toBe('function');
  });
});
