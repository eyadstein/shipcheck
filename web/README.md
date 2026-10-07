# Shipcheck dashboard

React and TypeScript dashboard for the Shipcheck API.

npm install
npm run dev # http://localhost:5173, API calls are proxied to 127.0.0.1:8787
npm test
npm run build


Set `VITE_API_URL` at build time when the dashboard is hosted on a different origin
than the API, and add that origin to `CORS_ORIGINS` on the server.
