// NELAIA Benchmark - Solar System Server - C Multi-threaded
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#pragma comment(lib, "ws2_32.lib")

const char* HTML = "HTTP/1.1 200 OK\r\n"
    "Content-Type: text/html\r\n"
    "Content-Length: 1094\r\n"
    "Connection: close\r\n\r\n"
    "<!DOCTYPE html><html><head><title>Solar</title><style>body{background:#000;margin:0;display:flex;justify-content:center;align-items:center;height:100vh}.c{position:relative;width:600px;height:600px}.sun{position:absolute;top:50%;left:50%;width:50px;height:50px;margin:-25px;background:#ff0;border-radius:50%;box-shadow:0 0 30px #ff0}.p{position:absolute;top:50%;left:50%;border-radius:50%}.m{width:6px;height:6px;background:#888;animation:r 2s linear infinite;--d:40px}.v{width:10px;height:10px;background:#da6;animation:r 3s linear infinite;--d:70px}.e{width:12px;height:12px;background:#48f;animation:r 4s linear infinite;--d:100px}.x{width:8px;height:8px;background:#f42;animation:r 6s linear infinite;--d:130px}.j{width:22px;height:22px;background:#db8;animation:r 10s linear infinite;--d:180px}@keyframes r{to{transform:rotate(1turn) translateX(var(--d))}}</style></head><body><div class=c><div class=sun></div><div class=\"p m\"></div><div class=\"p v\"></div><div class=\"p e\"></div><div class=\"p x\"></div><div class=\"p j\"></div></div></body></html>";

SOCKET g_sock;

DWORD WINAPI worker(LPVOID param) {
    char buf[512];
    while(1) {
        SOCKET client = accept(g_sock, 0, 0);
        if (client != INVALID_SOCKET) {
            recv(client, buf, 512, 0);
            send(client, HTML, 1188, 0);
            closesocket(client);
        }
    }
    return 0;
}

int main() {
    WSADATA wsa;
    WSAStartup(MAKEWORD(2,2), &wsa);
    
    g_sock = socket(AF_INET, SOCK_STREAM, 0);
    int opt = 1;
    setsockopt(g_sock, SOL_SOCKET, SO_REUSEADDR, (char*)&opt, sizeof(opt));
    
    struct sockaddr_in addr = {0};
    addr.sin_family = AF_INET;
    addr.sin_port = htons(8088);
    addr.sin_addr.s_addr = inet_addr("127.0.0.1");
    
    bind(g_sock, (struct sockaddr*)&addr, sizeof(addr));
    listen(g_sock, 8192);
    
    // Spawn 16 workers
    for (int i = 0; i < 16; i++) {
        CreateThread(NULL, 0, worker, NULL, 0, NULL);
    }
    
    // Main also works
    worker(NULL);
    return 0;
}
