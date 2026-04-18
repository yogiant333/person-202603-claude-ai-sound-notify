const express = require('express');
const http = require('http');
const path = require('path');
const { WebSocketServer } = require('ws');

const app = express();
const PORT = process.env.PORT || 9800;

// Middleware
app.use(express.json());

// CORS
app.use((req, res, next) => {
  res.header('Access-Control-Allow-Origin', '*');
  res.header('Access-Control-Allow-Headers', 'Content-Type');
  res.header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  if (req.method === 'OPTIONS') {
    return res.sendStatus(204);
  }
  next();
});

// Serve web directory as static files
app.use(express.static(path.join(__dirname, '..', 'web')));

// Validation constants
const VALID_SOURCES = ['claude-code', 'gemini', 'codex'];
const VALID_EVENTS = ['task_complete', 'need_input'];

// Health check
app.get('/api/health', (req, res) => {
  res.json({ status: 'ok' });
});

// Notification endpoint
app.post('/notify', (req, res) => {
  const { source, event, message, session_id } = req.body;

  if (!source || !VALID_SOURCES.includes(source)) {
    return res.status(400).json({
      error: `Invalid source. Must be one of: ${VALID_SOURCES.join(', ')}`
    });
  }

  if (!event || !VALID_EVENTS.includes(event)) {
    return res.status(400).json({
      error: `Invalid event. Must be one of: ${VALID_EVENTS.join(', ')}`
    });
  }

  const notification = {
    source,
    event,
    message: message || null,
    session_id: session_id || null,
    timestamp: new Date().toISOString()
  };

  console.log(`[NOTIFY] ${notification.source} - ${notification.event}`, notification);

  // Broadcast to all connected WebSocket clients
  wss.clients.forEach((client) => {
    if (client.readyState === 1) { // WebSocket.OPEN
      client.send(JSON.stringify(notification));
    }
  });

  res.json({ ok: true, notification });
});

// Create HTTP server and WebSocket server
const server = http.createServer(app);
const wss = new WebSocketServer({ server });

wss.on('connection', (ws) => {
  console.log('[WS] Client connected. Total:', wss.clients.size);
  ws.isAlive = true;
  ws.on('pong', () => { ws.isAlive = true; });
  ws.on('close', () => {
    console.log('[WS] Client disconnected. Total:', wss.clients.size);
  });
});

// Heartbeat: ping every 20s, terminate dead connections
setInterval(() => {
  wss.clients.forEach((ws) => {
    if (!ws.isAlive) return ws.terminate();
    ws.isAlive = false;
    ws.ping();
  });
}, 20000);

server.listen(PORT, '0.0.0.0', () => {
  console.log(`[ai-sound-notify] Server running on http://0.0.0.0:${PORT}`);
  console.log(`[ai-sound-notify] WebSocket available on ws://0.0.0.0:${PORT}`);
});
