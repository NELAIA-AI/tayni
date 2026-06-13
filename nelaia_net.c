#include <stdio.h>

// Esta es una función nativa de C, emulando la complejidad que el compilador ya NO tiene que hacer.
void tcp_listen(int port) {
    printf("[NELAIA EXTENSION: NETWORK]\n");
    printf("   Inicializando socket de C a bajo nivel...\n");
    printf("   Escuchando peticiones en el puerto: %d\n", port);
    printf("   (La complejidad técnica la maneja la libreria C, no el compilador LLVM de NELAIA)\n");
}
