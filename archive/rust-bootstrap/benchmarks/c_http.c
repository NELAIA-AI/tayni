// Minimal HTTP server for benchmark comparison - C
// Compile with: cl /O2 c_http.c ws2_32.lib /Fe:c_http.exe (MSVC)
// Or: gcc -O2 c_http.c -lws2_32 -o c_http.exe (MinGW)

#include <winsock2.h>
#include <stdio.h>

#pragma comment(lib, "ws2_32.lib")

int main() {
    WSADATA wsa;
    SOCKET server_fd, client_fd;
    struct sockaddr_in addr;
    char buffer[1024];
    const char *response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 23\r\n\r\n{\"benchmark\":\"c\",\"ok\":1}";
    
    WSAStartup(MAKEWORD(2,2), &wsa);
    
    server_fd = socket(AF_INET, SOCK_STREAM, 0);
    
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(38085);
    
    bind(server_fd, (struct sockaddr*)&addr, sizeof(addr));
    listen(server_fd, 1);
    
    printf("C HTTP Server listening on port 38085...\n");
    
    client_fd = accept(server_fd, NULL, NULL);
    recv(client_fd, buffer, 1024, 0);
    send(client_fd, response, strlen(response), 0);
    
    closesocket(client_fd);
    closesocket(server_fd);
    WSACleanup();
    
    return 0;
}
