import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';
import tailwindcss from '@tailwindcss/vite';
import metadata from './public/oauth-client-metadata.json' with { type: 'json' };

const SERVER_HOST = '127.0.0.1';
const SERVER_PORT = 12520;

export default defineConfig({
  plugins: [
    solid(),
    tailwindcss(),
    {
      name: 'skylynx-oauth-env',
      config(_config, { command }) {
        if (command === 'build') {
          process.env.VITE_OAUTH_CLIENT_ID = process.env.VITE_OAUTH_CLIENT_ID ?? metadata.client_id;
          process.env.VITE_OAUTH_REDIRECT_URI = process.env.VITE_OAUTH_REDIRECT_URI ?? metadata.redirect_uris[0];
          process.env.VITE_OAUTH_SCOPE = process.env.VITE_OAUTH_SCOPE ?? metadata.scope;
          return;
        }

        const redirectUri = `http://${SERVER_HOST}:${SERVER_PORT}${new URL(metadata.redirect_uris[0]).pathname}`;
        process.env.VITE_OAUTH_CLIENT_ID =
          `http://localhost?redirect_uri=${encodeURIComponent(redirectUri)}` +
          `&scope=${encodeURIComponent(metadata.scope)}`;
        process.env.VITE_OAUTH_REDIRECT_URI = redirectUri;
        process.env.VITE_OAUTH_SCOPE = metadata.scope;
      }
    }
  ],
  server: { host: SERVER_HOST, port: SERVER_PORT }
});
