#!/usr/bin/env python3
"""Minimal HTTP server for benchmark comparison"""
from http.server import HTTPServer, BaseHTTPRequestHandler
import json

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps({"benchmark": "python", "ok": 1}).encode())
    
    def log_message(self, format, *args):
        pass  # Suppress logging

if __name__ == '__main__':
    server = HTTPServer(('127.0.0.1', 38080), Handler)
    print('Python HTTP Server listening on port 38080...')
    server.handle_request()  # Handle one request then exit
