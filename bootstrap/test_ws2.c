#include <winsock2.h>
#pragma comment(lib, "ws2_32.lib")
int main() {
    WSADATA wsa;
    WSAStartup(0x0202, &wsa);
    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    closesocket(s);
    WSACleanup();
    return 42;
}
