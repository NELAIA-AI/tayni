import socket

HTML = b'''HTTP/1.1 200 OK\r
Content-Type: text/html; charset=utf-8\r
Content-Length: 347\r
Connection: close\r
\r
<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>NELAIA</title><style>body{font-family:system-ui;display:flex;justify-content:center;align-items:center;height:100vh;margin:0;background:#0a0a0a;color:#0f0}h1{font-size:3em}</style></head><body><h1>Python Server</h1></body></html>'''

sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind(('127.0.0.1', 8081))
sock.listen(10)

while True:
    client, addr = sock.accept()
    client.recv(1024)
    client.send(HTML)
    client.close()
