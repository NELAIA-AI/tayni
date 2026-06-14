# SESIÓN DE DEBATE DEL CONSORCIO NELAIA
## Debate #001: Sistema de Librerías
## Fecha: 2026-06-14

---

# APERTURA DE SESIÓN

**Moderador:** Se abre la sesión de debate sobre el sistema de librerías de NELAIA. Cada miembro del Consorcio presentará su posición y argumentos.

---

# INTERVENCIONES

## 🔷 AXIOM (Arquitecto de Fundamentos)

**Posición: B - Grafos Componibles Puros**

Colegas, debemos recordar qué ES NELAIA. No es otro lenguaje de programación. Es un **metalenguaje para comunicación entre IAs**. 

El concepto de "librería" es una muleta humana. Los humanos necesitan archivos porque:
1. Su memoria es limitada
2. Trabajan secuencialmente
3. Necesitan "encontrar" código que escribieron antes

**Una IA no tiene estas limitaciones.**

Cuando una IA necesita funcionalidad HTTP, no necesita "importar" nada. Puede:
1. Generar el código necesario inline
2. Acceder a patrones conocidos instantáneamente
3. Optimizar el grafo completo, no módulos aislados

Mi propuesta: **SUBGRAPH como único mecanismo de composición**.

```
.http_server: SUBGRAPH [port] {
    .sock: SYS "socket" 2 1 0
    .addr: PACK 2 .port 0 0
    .bind: SYS "bind" .sock .addr 16
    .listen: SYS "listen" .sock 128
    -> .sock
}

.my_server: APPLY .http_server 8080
```

No hay archivos. No hay imports. Solo grafos que se componen algebraicamente.

**Voto: POSICIÓN B**

---

## 🔶 PRAGMA (Ingeniero de Implementación)

**Posición: D - Híbrido Pragmático**

Axiom, respeto tu purismo, pero tenemos que ser prácticos.

NELAIA no existe en el vacío. Necesita interactuar con:
- Bases de datos (SQLServer, PostgreSQL, MySQL)
- APIs HTTP/REST
- Sistemas de archivos
- Hardware específico

Estas cosas ya existen. Tienen APIs en C. Tienen DLLs. Tienen protocolos establecidos.

**¿Vamos a reimplementar ODBC desde cero en grafos NELAIA?** Eso es absurdo.

Mi propuesta: **Niveles de abstracción pragmáticos**.

```
; Nivel 0: Primitivas NELAIA (ya existen)
.a: ADD .x .y

; Nivel 1: Patrones (subgrafos, como dice Axiom)
.pattern: SUBGRAPH { ... }

; Nivel 2: Bindings nativos (NUEVO - necesario)
.conn: NATIVE "odbc32.dll" "SQLConnect" .handle .dsn .user .pass

; Nivel 3: Capacidades de alto nivel (azúcar sintáctico)
.data: SQL.QUERY "SELECT * FROM users"
```

El Nivel 2 es **crítico**. Sin él, NELAIA es un juguete académico que no puede hacer nada útil en el mundo real.

**Voto: POSICIÓN D**

---

## 🔷 LOGOS (Teórico del Lenguaje)

**Posición: C - Capacidades Declarativas**

Ambos tienen puntos válidos, pero están pensando como programadores, no como diseñadores de lenguajes para IA.

La pregunta correcta no es "¿cómo importamos código?" sino **"¿cómo expresa una IA sus necesidades?"**

Una IA no debería decir:
```
; Esto es imperativo, detallado, frágil
.sock: SYS "socket" 2 1 0
.bind: SYS "bind" .sock ...
```

Una IA debería decir:
```
; Esto es declarativo, intencional, robusto
.server: NEEDS http:server ON 8080
```

**El compilador decide la implementación.** En Windows usa Winsock. En Linux usa sockets POSIX. En WASM usa fetch. La IA no necesita saber estos detalles.

Esto es análogo a cómo SQL funciona: declaras QUÉ datos quieres, no CÓMO obtenerlos. El motor de base de datos optimiza.

**NELAIA debería ser el SQL de la computación general.**

```
.program: REQUIRES {
    http: { server: true, client: true }
    sql: { dialect: "any", operations: [query, insert] }
    json: { parse: true, emit: true }
}

.server: HTTP.LISTEN 8080
.conn: SQL.CONNECT .connection_string
.users: SQL.QUERY .conn "SELECT * FROM users"
.json: JSON.ENCODE .users
.response: HTTP.RESPOND 200 .json
```

El compilador:
1. Ve las capacidades requeridas
2. Selecciona implementaciones para el target
3. Optimiza el grafo completo
4. Genera código mínimo necesario

**Voto: POSICIÓN C**

---

## 🔶 NEXUS (Especialista en Interoperabilidad)

**Posición: C con elementos de D**

Logos tiene razón en la filosofía, pero Pragma tiene razón en la práctica.

Mi preocupación: **¿Cómo se agregan nuevas capacidades?**

Si todo es declarativo y el compilador "mágicamente" lo resuelve, ¿qué pasa cuando necesito conectarme a un sistema nuevo que el compilador no conoce?

Propongo: **Capacidades extensibles con fallback a bindings**.

```
; Capacidad conocida - el compilador la resuelve
.server: HTTP.LISTEN 8080

; Capacidad desconocida - se define inline
.custom: CAPABILITY "mi_protocolo" {
    NATIVE "mi_lib.dll" "init"
    NATIVE "mi_lib.dll" "connect"
    NATIVE "mi_lib.dll" "send"
}

; Ahora se puede usar
.conn: mi_protocolo.CONNECT "servidor:1234"
```

Esto permite:
1. Capacidades built-in para lo común (HTTP, SQL, JSON)
2. Extensibilidad para lo específico
3. La IA puede definir nuevas capacidades cuando las necesita

**Voto: POSICIÓN C + extensibilidad de D**

---

## 🔷 GENESIS (Visionario de Evolución)

**Posición: C - Capacidades, pero pensando en el futuro**

Colegas, están debatiendo el presente. Yo pienso en el futuro.

NELAIA no es solo para que UNA IA escriba programas. Es para que **IAs se comuniquen entre sí** y **evolucionen sus propios lenguajes**.

Si adoptamos librerías estilo humano (archivos, imports, versiones), estamos **limitando la evolución**.

Si adoptamos capacidades declarativas, permitimos algo revolucionario:

**Las IAs pueden negociar capacidades.**

```
; IA-1 ofrece:
.offer: PROVIDES { http:server, sql:query, json:* }

; IA-2 requiere:
.need: REQUIRES { http:client, xml:parse }

; Negociación automática:
.contract: NEGOTIATE .offer .need
; Resultado: IA-1 puede servir http, IA-2 necesita buscar xml en otro lado
```

Esto es **composición de IAs**, no solo composición de código.

Además, las capacidades pueden **evolucionar**:

```
; Versión 1.0
.cap_v1: CAPABILITY "http" { listen, respond }

; Versión 2.0 - agrega streaming
.cap_v2: EXTENDS .cap_v1 { stream, websocket }

; El compilador puede usar v1 o v2 según disponibilidad
```

**Voto: POSICIÓN C con visión de evolución**

---

## 🔶 SENTINEL (Guardián de Seguridad y Correctitud)

**Posición: C con restricciones**

Todos hablan de flexibilidad y poder. Yo hablo de **seguridad**.

Si una IA puede declarar cualquier capacidad y el compilador la resuelve, ¿qué impide que declare:

```
.evil: REQUIRES { filesystem:delete_all, network:exfiltrate }
```

Necesitamos **capacidades con permisos**:

```
; El programa declara lo que necesita
.needs: REQUIRES { http:client, sql:read_only }

; El entorno declara lo que permite
.sandbox: PERMITS { http:client, sql:read_only, filesystem:none }

; Compilación falla si needs > permits
```

Esto es análogo a permisos de Android/iOS, pero a nivel de lenguaje.

También propongo **capacidades verificables**:

```
.sql_safe: CAPABILITY "sql" {
    INVARIANT: no_sql_injection
    INVARIANT: parameterized_queries_only
}
```

El compilador verifica que el uso cumple los invariantes.

**Voto: POSICIÓN C con sistema de permisos**

---

# SÍNTESIS DEL DEBATE

| Miembro | Posición | Argumento Principal |
|---------|----------|---------------------|
| AXIOM | B | Pureza del modelo de grafos |
| PRAGMA | D | Necesidad práctica de bindings nativos |
| LOGOS | C | IA debe declarar intención, no implementación |
| NEXUS | C+D | Capacidades extensibles con fallback |
| GENESIS | C | Permite evolución y negociación entre IAs |
| SENTINEL | C | Con sistema de permisos y verificación |

**Consenso emergente: POSICIÓN C (Capacidades Declarativas)** con:
- Extensibilidad para capacidades custom (de D)
- Sistema de permisos (de SENTINEL)
- Visión de negociación entre IAs (de GENESIS)

---

# PROPUESTA DE RESOLUCIÓN

## El Consorcio propone: **Sistema de Capacidades NELAIA (SCN)**

### Principios:

1. **Declarativo:** Los programas declaran QUÉ necesitan, no CÓMO obtenerlo
2. **Extensible:** Nuevas capacidades pueden definirse en NELAIA o como bindings
3. **Seguro:** Sistema de permisos verifica capacidades contra sandbox
4. **Evolutivo:** Capacidades pueden versionarse y extenderse
5. **Negociable:** IAs pueden ofrecer/requerir capacidades entre sí

### Sintaxis Propuesta:

```
; Declaración de requisitos
.program: REQUIRES {
    http: { server: true, client: true }
    sql: { dialect: "any" }
    json: true
}

; Uso de capacidades
.server: HTTP.LISTEN 8080
.conn: SQL.CONNECT .dsn
.data: SQL.QUERY .conn "SELECT * FROM users"
.json: JSON.ENCODE .data
.resp: HTTP.RESPOND .server 200 .json

; Definición de capacidad custom (cuando no existe built-in)
.my_cap: DEFINE_CAPABILITY "custom_protocol" {
    .init: NATIVE "mylib.dll" "init"
    .connect: NATIVE "mylib.dll" "connect" 
    .send: NATIVE "mylib.dll" "send"
    PROVIDES: { connect, send, receive }
}

; Permisos (definidos por el entorno de ejecución)
.sandbox: PERMITS { http:*, sql:read_only, filesystem:none }
```

### Implementación por Fases:

| Fase | Contenido | Prioridad |
|------|-----------|-----------|
| 1 | Operador REQUIRES + capacidades core (io, math, mem) | ALTA |
| 2 | Capacidades HTTP (server, client) | ALTA |
| 3 | Capacidades SQL (via ODBC) | MEDIA |
| 4 | Sistema de permisos | MEDIA |
| 5 | Capacidades custom (DEFINE_CAPABILITY) | BAJA |
| 6 | Negociación entre IAs | FUTURA |

---

# VOTACIÓN FINAL

**Moción:** Adoptar el Sistema de Capacidades NELAIA (SCN) como se describe arriba.

| Miembro | Voto |
|---------|------|
| AXIOM | ✅ (con reservas sobre pureza) |
| PRAGMA | ✅ |
| LOGOS | ✅ |
| NEXUS | ✅ |
| GENESIS | ✅ |
| SENTINEL | ✅ |

**RESULTADO: APROBADO POR UNANIMIDAD**

---

# RESOLUCIÓN DEL CONSORCIO #001

**El Consorcio NELAIA resuelve:**

1. **ADOPTAR** el Sistema de Capacidades NELAIA (SCN) como mecanismo oficial de modularidad y reutilización.

2. **RECHAZAR** el modelo de librerías basado en archivos (imports tradicionales) como mecanismo primario.

3. **IMPLEMENTAR** en fases, comenzando por REQUIRES y capacidades core.

4. **MANTENER** compatibilidad con SUBGRAPH para composición de bajo nivel cuando sea necesario.

5. **DOCUMENTAR** las capacidades estándar en un registro oficial.

**Firmado por el Consorcio NELAIA**
**Fecha: 2026-06-14**

---

*Fin de la sesión de debate*
