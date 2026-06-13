const http = require('http');

const HTML = `<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>NELAIA</title><style>body{font-family:system-ui;display:flex;justify-content:center;align-items:center;height:100vh;margin:0;background:#0a0a0a;color:#0f0}h1{font-size:3em}</style></head><body><h1>Node.js Server</h1></body></html>`;

const server = http.createServer((req, res) => {
    res.writeHead(200, {
        'Content-Type': 'text/html; charset=utf-8',
        'Content-Length': 347,
        'Connection': 'close'
    });
    res.end(HTML);
});

server.listen(8082, '127.0.0.1');
