#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "ws2_32.lib")
void __attribute__((noinline)) server() {
    WSADATA wsa;
    WSAStartup(0x0202, &wsa);
    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    struct sockaddr_in addr = {0};
    addr.sin_family = AF_INET;
    addr.sin_port = htons(9999);
    addr.sin_addr.s_addr = INADDR_ANY;
    int r = bind(s, (struct sockaddr*)&addr, sizeof(addr));
    if (r != 0) { WSACleanup(); return; }
    listen(s, 1);
    SOCKET c = accept(s, NULL, NULL);
    const char* msg = "Hello from C!";
    send(c, msg, 13, 0);
    closesocket(c);
    closesocket(s);
    WSACleanup();
}
int main() { server(); return 0; }
