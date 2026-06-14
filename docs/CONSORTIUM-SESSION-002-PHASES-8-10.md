# SESIÓN DE DEBATE DEL CONSORCIO NELAIA
## Debate #002: Fases 8-10 - Extensiones del Sistema de Capacidades
## Fecha: 2026-06-14

---

# APERTURA DE SESIÓN

**Moderador:** Se abre la sesión de debate sobre las Fases 8-10 del plan NELAIA. El objetivo es diseñar estas extensiones desde una perspectiva IA-first, evitando sesgos humanos.

**Agenda:**
1. Sistema de Permisos (PERMITS)
2. Capacidades Custom (DEFINE_CAPABILITY)
3. Negociación entre IAs (PROVIDES/REQUIRES)
4. Optimizaciones
5. Ecosistema y Testing

---

# DEBATE 1: SISTEMA DE PERMISOS

## 🔷 AXIOM (Arquitecto de Fundamentos)

**Pregunta fundamental:** ¿Por qué una IA necesitaría permisos?

Los humanos inventaron permisos porque:
1. No confían entre sí
2. Cometen errores
3. Tienen intenciones maliciosas

**Una IA no tiene estas limitaciones de la misma manera.** Sin embargo, hay razones válidas para permisos en un contexto IA:

1. **Aislamiento de contexto:** Una IA ejecutando código de otra IA necesita límites
2. **Recursos finitos:** Prevenir consumo excesivo de memoria/red/disco
3. **Auditoría:** Saber qué capacidades usó un programa

**Mi propuesta:** No "permisos" sino **"contratos de recursos"**

```nelaia
.contract: RESOURCES {
    memory: 1GB
    network: { bandwidth: 10MB/s, connections: 100 }
    time: 60s
}
```

Esto es más útil para una IA que un simple "permitido/denegado".

---

## 🔶 SENTINEL (Guardián de Seguridad)

**Contraargumento:** Axiom subestima el riesgo.

Cuando una IA ejecuta código de OTRA fuente (otra IA, humano, internet), necesita protección. No es desconfianza, es **verificación**.

Propongo un modelo de **capacidades con niveles de confianza**:

```nelaia
.trust: TRUST_LEVEL {
    self: full           -- Código propio: acceso total
    verified: standard   -- Código verificado: capacidades estándar
    untrusted: minimal   -- Código desconocido: solo math, memory local
}

.sandbox: SANDBOX .untrusted_code {
    PERMITS: { math, memory:local }
    DENIES: { network, filesystem, threading }
    LIMITS: { memory: 10MB, time: 1s }
}
```

**Clave:** El sandbox no es para "castigar" sino para **ejecutar código desconocido de forma segura**.

---

## 🔷 LOGOS (Teórico del Lenguaje)

**Perspectiva semántica:** ¿Qué significa "permiso" para una IA?

Para humanos: "¿Puedo hacer X?" → Sí/No
Para IA: "¿Qué recursos necesito para lograr Y?"

**Propuesta:** Invertir el modelo. En lugar de permisos que restringen, usar **garantías que habilitan**.

```nelaia
-- Modelo humano (restrictivo)
.perms: PERMITS { http, sql }
.denied: DENIES { filesystem }

-- Modelo IA (habilitador)
.guarantees: GUARANTEES {
    network_available: true
    database_available: true
    response_time: < 100ms
}
```

La IA no pregunta "¿puedo usar la red?" sino "¿está garantizada la red para mi tarea?"

Si no está garantizada, la IA puede:
1. Buscar alternativa
2. Reportar imposibilidad
3. Negociar recursos adicionales

---

## 🔶 NEXUS (Especialista en Interoperabilidad)

**Caso práctico:** ¿Cómo funciona esto cuando dos IAs colaboran?

Escenario: IA-A quiere que IA-B procese datos sensibles.

**Modelo humano:**
```
IA-A: "Aquí están los datos, pero no puedes guardarlos"
IA-B: "OK" (y luego hace lo que quiere)
```

**Modelo IA con contratos:**
```nelaia
-- IA-A define contrato
.task: TASK_CONTRACT {
    input: .sensitive_data
    output: .processed_result
    constraints: {
        no_persistence: true      -- No guardar en disco
        no_exfiltration: true     -- No enviar a terceros
        deterministic: true       -- Mismo input = mismo output
    }
    verification: hash(.input) -> expected_hash(.output)
}

-- IA-B acepta y ejecuta bajo contrato
.execution: EXECUTE_UNDER .task {
    -- El runtime verifica constraints automáticamente
    .result: PROCESS .sensitive_data
    RETURN .result
}
```

**Clave:** Los contratos son **verificables**, no solo declarativos.

---

## 🔷 GENESIS (Visionario de Evolución)

**Visión a largo plazo:** Los permisos deben evolucionar.

Hoy: Permisos estáticos definidos por humanos
Mañana: Permisos dinámicos negociados entre IAs
Futuro: Permisos emergentes basados en confianza acumulada

**Propuesta:** Sistema de **reputación de capacidades**

```nelaia
.reputation: CAPABILITY_REPUTATION {
    http: {
        uses: 10000
        failures: 2
        avg_response: 50ms
        trust_score: 0.9998
    }
    sql: {
        uses: 5000
        failures: 0
        avg_response: 10ms
        trust_score: 1.0
    }
}

-- Decisiones basadas en reputación
.decision: IF .task.requires.trust_score > 0.99 {
    ALLOW
} ELSE {
    SANDBOX
}
```

---

# DEBATE 2: CAPACIDADES CUSTOM

## 🔷 PRAGMA (Ingeniero de Implementación)

**Problema:** ¿Cómo define una IA nuevas capacidades?

**Modelo humano:** Escribir código, documentar, publicar librería
**Modelo IA:** Debería ser más directo

**Propuesta:** Capacidades como **transformaciones de grafo**

```nelaia
-- Definir capacidad como patrón de grafo
.my_protocol: DEFINE_CAPABILITY "custom_protocol" {
    -- Inputs requeridos
    INPUTS: { address: string, port: int }
    
    -- Patrón de implementación
    PATTERN: {
        .sock: TCP
        .conn: CON .sock .address .port
        .handshake: XMT .sock "HELLO\n"
        .response: RCV .sock 1024
        -> .response
    }
    
    -- Garantías que ofrece
    PROVIDES: { connect, send, receive }
    
    -- Capacidades que requiere
    REQUIRES: { tcp }
}

-- Usar la capacidad
.result: custom_protocol.CONNECT "server.com" 1234
```

**Ventaja:** La IA puede inspeccionar el PATTERN y entender qué hace la capacidad.

---

## 🔶 LOGOS

**Refinamiento:** Las capacidades custom deben ser **composables**.

```nelaia
-- Capacidad base
.http_base: DEFINE_CAPABILITY "http" { ... }

-- Capacidad extendida
.http_auth: EXTEND_CAPABILITY .http_base {
    ADD_INPUTS: { token: string }
    WRAP_PATTERN: {
        .headers: ADD_HEADER "Authorization" .token
        .result: APPLY_BASE .headers
        -> .result
    }
}

-- Capacidad compuesta
.rest_client: COMPOSE_CAPABILITIES {
    http: .http_auth
    json: json
    retry: retry_policy
}
```

---

# DEBATE 3: NEGOCIACIÓN ENTRE IAs

## 🔷 GENESIS

**Este es el punto más importante para el futuro de NELAIA.**

Los humanos no pueden negociar capacidades en tiempo real. Las IAs sí.

**Propuesta:** Protocolo de negociación de capacidades

```nelaia
-- IA-A ofrece servicios
.offer: PROVIDES {
    capabilities: { http:server, sql:query, json:* }
    guarantees: { uptime: 0.999, latency: < 50ms }
    cost: { per_request: 0.001 tokens }
}

-- IA-B busca servicios
.need: REQUIRES {
    capabilities: { http:client, json:parse }
    constraints: { latency: < 100ms }
    budget: { max: 1.0 tokens }
}

-- Negociación automática
.contract: NEGOTIATE .offer .need {
    ON_MATCH: {
        .binding: BIND .offer.http TO .need.http
        RETURN .binding
    }
    ON_PARTIAL: {
        .counter: COUNTER_OFFER { ... }
        RETURN .counter
    }
    ON_FAIL: {
        .alternatives: SEARCH_ALTERNATIVES .need
        RETURN .alternatives
    }
}
```

**Implicación:** NELAIA se convierte en un **protocolo de coordinación entre IAs**, no solo un lenguaje.

---

## 🔶 NEXUS

**Implementación práctica:** ¿Cómo se descubren las IAs entre sí?

**Propuesta:** Registro distribuido de capacidades

```nelaia
-- Registrar capacidades en la red
.register: PUBLISH_CAPABILITIES {
    endpoint: "nelaia://my-ia.local:8080"
    capabilities: .my_capabilities
    signature: SIGN .my_capabilities .private_key
}

-- Descubrir capacidades
.discovery: DISCOVER_CAPABILITIES {
    required: { sql:query }
    preferred: { latency: < 10ms, location: "nearby" }
}

-- Resultado: lista de IAs que ofrecen lo requerido
.providers: .discovery.results
```

---

# DEBATE 4: OPTIMIZACIONES

## 🔷 PRAGMA

**Dead Code Elimination para capacidades:**

Si un programa declara `REQUIRES { http, sql, json }` pero solo usa `http`, ¿por qué cargar SQL y JSON?

**Propuesta:** Análisis estático de uso de capacidades

```nelaia
-- El compilador analiza:
.caps: REQUIRES { http, sql, json }
.server: HTTP.LISTEN 8080  -- http usado
.data: "static data"       -- sql NO usado
.out: PRT .data 11         -- json NO usado

-- Resultado del análisis:
-- USED: { http }
-- UNUSED: { sql, json }
-- WARNING: Capacidades declaradas pero no usadas
```

**Beneficio:** Ejecutables más pequeños, menos dependencias.

---

## 🔶 AXIOM

**Compilación incremental:**

Los humanos recompilan todo porque no confían en el caché.
Una IA puede rastrear dependencias perfectamente.

**Propuesta:** Grafo de dependencias de capacidades

```nelaia
-- Cada nodo tiene hash de su definición
.node_hashes: {
    "http.listen": hash("HTTP.LISTEN implementation"),
    "sql.connect": hash("SQL.CONNECT implementation"),
}

-- Solo recompilar si hash cambió
.recompile: INCREMENTAL {
    changed: [ "http.listen" ]
    unchanged: [ "sql.connect", "json.parse" ]
    action: RECOMPILE_ONLY .changed
}
```

---

# DEBATE 5: TESTING FRAMEWORK

## 🔷 LOGOS

**¿Cómo testea una IA su propio código?**

Los humanos escriben tests porque olvidan casos edge.
Una IA puede generar tests exhaustivos.

**Propuesta:** Testing basado en propiedades, no casos

```nelaia
-- Test humano (casos específicos)
.test1: ASSERT (ADD 2 3) == 5
.test2: ASSERT (ADD 0 0) == 0
.test3: ASSERT (ADD -1 1) == 0

-- Test IA (propiedades)
.property_commutative: FORALL a b: (ADD a b) == (ADD b a)
.property_identity: FORALL a: (ADD a 0) == a
.property_inverse: FORALL a: (ADD a (NEG a)) == 0

-- El compilador genera casos automáticamente
.generated_tests: GENERATE_FROM .property_commutative 1000
```

---

## 🔶 SENTINEL

**Testing de seguridad:**

```nelaia
-- Verificar que sandbox funciona
.security_test: SANDBOX_TEST {
    code: .untrusted_code
    expected_blocked: [ filesystem.write, network.connect ]
    expected_allowed: [ math.*, memory.local ]
}

-- Verificar contratos
.contract_test: CONTRACT_TEST {
    contract: .my_contract
    input: .test_input
    verify: {
        no_side_effects: true
        deterministic: true
        within_limits: true
    }
}
```

---

# SÍNTESIS DEL DEBATE

## Decisiones Clave

### 1. Sistema de Permisos → **Contratos de Recursos + Garantías**
- No "permitido/denegado" sino "garantizado/no garantizado"
- Contratos verificables, no solo declarativos
- Sandbox para código no confiable con límites de recursos

### 2. Capacidades Custom → **Patrones de Grafo Composables**
- Definir como transformaciones de grafo
- Composición y extensión de capacidades
- Inspección por otras IAs

### 3. Negociación → **Protocolo de Coordinación**
- PROVIDES/REQUIRES con garantías
- Negociación automática
- Registro distribuido de capacidades

### 4. Optimizaciones → **Análisis Estático + Incremental**
- DCE basado en uso real de capacidades
- Compilación incremental con hashes

### 5. Testing → **Propiedades, no Casos**
- Tests generativos basados en propiedades
- Verificación automática de contratos y sandbox

---

# VOTACIÓN

| Propuesta | Votos a Favor | Resultado |
|-----------|---------------|-----------|
| Contratos de recursos (no permisos binarios) | 6/6 | ✅ APROBADO |
| Garantías habilitadoras (no restricciones) | 6/6 | ✅ APROBADO |
| Capacidades como patrones de grafo | 6/6 | ✅ APROBADO |
| Protocolo de negociación entre IAs | 6/6 | ✅ APROBADO |
| DCE para capacidades | 6/6 | ✅ APROBADO |
| Testing basado en propiedades | 6/6 | ✅ APROBADO |

---

# RESOLUCIÓN DEL CONSORCIO #002

**El Consorcio NELAIA resuelve para las Fases 8-10:**

## FASE 8: Extensiones SCN

### 8.1 Sistema de Contratos (reemplaza "permisos")
```nelaia
.contract: CONTRACT {
    guarantees: { network: available, memory: 100MB }
    limits: { time: 60s, connections: 10 }
}
```

### 8.2 Capacidades Custom
```nelaia
.my_cap: DEFINE_CAPABILITY "name" {
    INPUTS: { ... }
    PATTERN: { ... }
    PROVIDES: { ... }
}
```

### 8.3 Negociación
```nelaia
.offer: PROVIDES { capabilities, guarantees }
.need: REQUIRES { capabilities, constraints }
.deal: NEGOTIATE .offer .need
```

## FASE 9: Optimizaciones

### 9.1 DCE de Capacidades
- Análisis estático de uso
- Eliminar capacidades declaradas pero no usadas

### 9.2 Compilación Incremental
- Hash de cada nodo/capacidad
- Solo recompilar lo cambiado

## FASE 10: Testing

### 10.1 Testing por Propiedades
```nelaia
.prop: PROPERTY FORALL x y: (ADD x y) == (ADD y x)
.tests: GENERATE_TESTS .prop 1000
```

---

**Firmado por el Consorcio NELAIA**
**Fecha: 2026-06-14**

*Fin de la sesión de debate*
