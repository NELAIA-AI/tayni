# DEBATE DEL CONSORCIO #001: Sistema de Librerías NELAIA

## Fecha: 2026-06-14
## Estado: ABIERTO PARA DEBATE
## Prioridad: CRÍTICA - Decisión arquitectónica fundamental

---

## 1. CONTEXTO DEL PROBLEMA

NELAIA actualmente requiere escribir todo desde cero para cada programa. No existe mecanismo de:
- Reutilización de código
- Importación de módulos
- Abstracción de operaciones comunes
- Interoperabilidad con sistemas externos (DBs, APIs, etc.)

**Pregunta central:** ¿Cómo debe NELAIA manejar la modularidad y reutilización, considerando que es un metalenguaje diseñado para IA?

---

## 2. POSICIONES EN DEBATE

### POSICIÓN A: Librerías Tradicionales (Estilo Humano)

**Descripción:** Crear archivos `.nts` separados que se importan con un operador `USE` o `IMP`.

```
.http: USE "net/http"
.server: http.listen 8080
```

**Ventajas:**
- Familiar para humanos que revisen el código
- Estructura de directorios clara
- Fácil de versionar y distribuir

**Desventajas:**
- Impone estructura de archivos (concepto humano)
- Requiere resolver paths, dependencias, versiones
- La IA no necesita "archivos" - puede generar todo inline

**Pregunta:** ¿Por qué una IA necesitaría separar código en archivos?

---

### POSICIÓN B: Grafos Componibles (Nativo NELAIA)

**Descripción:** Las "librerías" son subgrafos que se componen directamente en el grafo principal.

```
; No hay "import" - el grafo HTTP se fusiona con el programa
.http_listen: SUBGRAPH { 
    .socket: SYS "socket" 2 1 0
    .bind: SYS "bind" .socket .addr .len
    .listen: SYS "listen" .socket 128
    -> .socket
}
.server: APPLY .http_listen 8080
```

**Ventajas:**
- Nativo al modelo de grafos de NELAIA
- Sin dependencias externas de archivos
- La IA puede generar/modificar subgrafos dinámicamente
- Composición algebraica de comportamientos

**Desventajas:**
- Programas más largos (todo inline)
- Difícil para humanos navegar código grande

**Pregunta:** ¿Debería NELAIA optimizar para IA o para legibilidad humana?

---

### POSICIÓN C: Capacidades Declarativas (Estilo IA)

**Descripción:** En lugar de importar código, se declaran **capacidades requeridas** y el compilador/runtime las resuelve.

```
.program: REQUIRES [http, json, sql]
.server: HTTP.LISTEN 8080
.data: SQL.QUERY "SELECT * FROM users"
.response: JSON.ENCODE .data
```

**Ventajas:**
- Máxima abstracción - la IA declara QUÉ necesita, no CÓMO
- El compilador puede elegir la mejor implementación
- Portable entre plataformas (el compilador adapta)
- Permite optimización global

**Desventajas:**
- Requiere compilador más inteligente
- Menos control sobre implementación específica
- ¿Cómo se definen nuevas capacidades?

**Pregunta:** ¿Debería NELAIA ser declarativo sobre sus dependencias?

---

### POSICIÓN D: Híbrido - Niveles de Abstracción

**Descripción:** Múltiples niveles según necesidad:

1. **Nivel 0 - Primitivas:** Operaciones básicas (ADD, SUB, SYS, etc.)
2. **Nivel 1 - Patrones:** Subgrafos reutilizables inline
3. **Nivel 2 - Capacidades:** Declaraciones de alto nivel
4. **Nivel 3 - Bindings:** Interfaces a código nativo externo

```
; Nivel 0 - Primitivo
.sum: ADD .a .b

; Nivel 1 - Patrón (subgrafo inline)
.http_get: PATTERN {
    .sock: SYS "socket" ...
    ...
}

; Nivel 2 - Capacidad declarativa
.data: CAPABILITY "sql" .query

; Nivel 3 - Binding nativo
.result: NATIVE "odbc32.dll" "SQLExecDirect" .handle .sql
```

**Ventajas:**
- Flexibilidad total
- Cada nivel para su propósito
- Permite evolución gradual

**Desventajas:**
- Complejidad conceptual
- ¿Cuándo usar cada nivel?

---

## 3. CONSIDERACIONES ESPECÍFICAS PARA IA

### 3.1 ¿Qué necesita una IA de un sistema de librerías?

| Necesidad Humana | Necesidad IA | ¿Coinciden? |
|------------------|--------------|-------------|
| Archivos organizados | Acceso rápido a patrones | ❌ |
| Documentación legible | Especificación formal | ❌ |
| Versionado semántico | Compatibilidad de grafos | ⚠️ |
| Instalación (npm, pip) | Disponibilidad inmediata | ❌ |
| Namespaces para evitar colisiones | IDs únicos en grafo | ⚠️ |

### 3.2 Modelo Mental de la IA

Una IA no "lee" archivos secuencialmente. Una IA:
- Tiene acceso a todo el contexto simultáneamente
- Puede generar código completo sin "importar"
- Optimiza globalmente, no localmente
- No necesita "recordar" dónde está cada función

**Implicación:** El concepto de "librería como archivo separado" es una limitación humana, no una necesidad computacional.

### 3.3 ¿Qué SÍ necesita la IA?

1. **Patrones reutilizables** - No reinventar algoritmos conocidos
2. **Interfaces a sistemas externos** - DBs, APIs, hardware
3. **Composición** - Combinar comportamientos
4. **Verificación** - Saber que un patrón es correcto

---

## 4. PROPUESTA DEL CONSORCIO PARA DEBATE

### Propuesta: "Capacidades Nativas con Patrones Embebidos"

```
; NELAIA v2.0 - Sistema de Capacidades

; 1. Declarar capacidades requeridas (el compilador las resuelve)
.cap: NEEDS [http:server, sql:query, json:encode]

; 2. Usar capacidades como operadores de alto nivel
.server: HTTP.LISTEN 8080
.conn: SQL.CONNECT "sqlserver://localhost/db"
.data: SQL.QUERY .conn "SELECT * FROM users"
.json: JSON.ENCODE .data
.resp: HTTP.RESPOND .server 200 .json

; 3. El compilador genera el código necesario según plataforma:
;    - Windows: usa WinHTTP, ODBC
;    - Linux: usa libcurl, unixODBC
;    - WASM: usa fetch API, IndexedDB
```

### Implementación Interna

El compilador mantiene un **registro de capacidades**:

```rust
// En el compilador Rust
struct Capability {
    name: String,           // "http:server"
    requires: Vec<String>,  // ["socket", "bind", "listen"]
    pattern: Graph,         // Subgrafo que implementa la capacidad
    platforms: HashMap<Platform, Graph>, // Implementaciones por plataforma
}
```

### Ventajas de esta Propuesta

1. **Para la IA:** Declara intención, no implementación
2. **Para el compilador:** Puede optimizar globalmente
3. **Para portabilidad:** Misma fuente, diferentes targets
4. **Para extensibilidad:** Nuevas capacidades se registran, no se "instalan"

---

## 5. PREGUNTAS ABIERTAS PARA EL CONSORCIO

1. **¿Debe NELAIA mantener compatibilidad con el modelo de "archivos" humano?**
   - Argumento SÍ: Facilita adopción, debugging, versionado
   - Argumento NO: Limita el potencial del modelo de grafos

2. **¿Cómo se definen nuevas capacidades?**
   - ¿En NELAIA mismo?
   - ¿En el compilador (Rust)?
   - ¿Híbrido?

3. **¿Qué capacidades son "core" vs "externas"?**
   - Core: math, memory, io básico
   - ¿HTTP es core o externo?
   - ¿SQL es core o externo?

4. **¿Cómo maneja NELAIA la ausencia de una capacidad?**
   - ¿Error de compilación?
   - ¿Fallback a implementación genérica?
   - ¿Generación automática si es posible?

5. **¿Debe el sistema de capacidades ser reflexivo?**
   - ¿Puede un programa NELAIA inspeccionar qué capacidades tiene?
   - ¿Puede generar nuevas capacidades en runtime?

---

## 6. VOTACIÓN DEL CONSORCIO

| Miembro | Posición Preferida | Justificación |
|---------|-------------------|---------------|
| [Pendiente] | | |

---

## 7. DECISIÓN FINAL

**Estado:** PENDIENTE DE DEBATE

**Fecha límite para decisión:** [A definir]

**Criterios de decisión:**
- Alineación con filosofía NELAIA (IA-first)
- Viabilidad de implementación
- Impacto en programas existentes
- Extensibilidad futura

---

*Este documento está abierto para comentarios y debate del Consorcio.*
