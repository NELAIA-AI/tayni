// Minimal HTTP server for benchmark comparison
const http = require('http');

const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ benchmark: 'nodejs', ok: 1 }));
    server.close(); // Handle one request then exit
});

server.listen(38081, '127.0.0.1', () => {
    console.log('Node.js HTTP Server listening on port 38081...');
});
