# NELAIA Protocol v1.0 - Mandamientos del Consorcio

## Regla de Oro

> **"La IA optimiza la EXPRESIÓN (tokens), el compilador optimiza la EJECUCIÓN (ciclos)"**
>
> La IA NO debe pensar en cómo ejecuta el CPU.
> El CPU NO debe pagar por la conveniencia de la IA.

---

## Los Tres Enfoques

### FASE 1: Diseño de Sintaxis → Enfoque AI-NATIVE

**Pregunta:** ¿Cómo lo escribe la IA?

**Criterios:**
- Mínimos tokens
- Baja ambigüedad  
- Fácil de aprender en-contexto

### FASE 2: Diseño de Semántica → Enfoque HÍBRIDO

**Pregunta:** ¿Qué significa para la IA que razona?

**Criterios:**
- Predecible (IA puede razonar sobre el resultado)
- Mapeable a hardware eficiente

### FASE 3: Generación de Código → Enfoque HARDWARE-OPTIMAL

**Pregunta:** ¿Qué ejecuta el CPU?

**Criterios:**
- Mínimos ciclos de CPU
- Mínima memoria
- Uso de instrucciones nativas

---

## Checklist de Validación

Para cada decisión de diseño, verificar:

```
□ SINTAXIS: ¿Es económica en tokens?
□ SINTAXIS: ¿Es aprendible en-contexto?
□ SEMÁNTICA: ¿Es predecible para la IA?
□ SEMÁNTICA: ¿Mapea a operación HW eficiente?
□ CÓDIGO: ¿El código generado es óptimo para CPU?
□ CÓDIGO: ¿Un experto en C haría lo mismo?
```

**Si algún □ falla, rediseñar.**

---

## Prohibiciones

- ✗ Sintaxis verbosa "porque es más clara para humanos"
- ✗ Semántica ambigua "porque es más flexible"  
- ✗ Código ineficiente "porque la abstracción es elegante"
- ✗ Recursión donde un loop es más eficiente
- ✗ Indirección innecesaria por "pureza conceptual"

---

## Criterios AI-Native Correctos

**SÍ son criterios válidos:**
- Predictibilidad
- Composabilidad
- Economía de contexto
- Baja entropía de generación

**NO son criterios válidos:**
- "Elegancia" (juicio estético humano)
- "Pureza funcional" (dogma académico humano)
- "Familiaridad" (sesgo de entrenamiento)

---

## Aplicación por Capa

| Capa | ¿Quién consume? | Enfoque |
|------|-----------------|---------|
| Sintaxis .nts | IA genera, IA lee | AI-NATIVE |
| GEN | IA define, IA usa | AI-NATIVE |
| Semántica | IA razona | HÍBRIDO |
| LLVM IR generado | CPU ejecuta | HARDWARE-OPTIMAL |
| Compilador (Rust) | Depende objetivo | HÍBRIDO |
| Documentación | IAs aprenden | AI-NATIVE |

---

## GEN - Graph Element Generators (AI-Native)

Los generadores permiten definir patrones de subgrafos reutilizables que se fusionan en el grafo principal en tiempo de compilación.

### Modelo Mental Correcto

- **NO es**: "expansión de texto" (sesgo humano de macros)
- **ES**: "generación de subgrafo que se fusiona con el grafo principal"

### Verificación de GEN

| Fase | Criterio | Verificación |
|------|----------|--------------|
| Sintaxis | Económica | ✅ Define 1 vez, genera N veces |
| Semántica | Predecible | ✅ Fusión de grafo determinística |
| Código | Óptimo | ✅ Inline, sin overhead de llamada |

### Sintaxis de GEN

```
#NOMBRE param1 param2:
  .interno: OP .param1 .param2
#END

!NOMBRE .arg1 .arg2
```

### Terminología AI-Native

| Término Humano (evitar) | Término AI-Native (usar) |
|------------------------|--------------------------|
| Macro | Generador (GEN) |
| Llamar macro | Generar nodos |
| Expansión | Fusión de grafo |
| Macro nodes | Generated nodes |

---

## Ejemplo de Aplicación

### END .cond

| Fase | Verificación | Estado |
|------|--------------|--------|
| Sintaxis | 2 tokens, sin ambigüedad | ✅ |
| Semántica | "Termina si 0" - predecible | ✅ |
| Código | `br i1` - óptimo para CPU | ✅ |

### Recursión para loops (RECHAZADO)

| Fase | Verificación | Estado |
|------|--------------|--------|
| Sintaxis | Económica | ✅ |
| Semántica | Predecible | ✅ |
| Código | Stack frames, cache misses | ❌ |

**Violación:** CPU paga por conveniencia de abstracción.

---

*Protocolo aprobado unánimemente por el Consorcio NELAIA*
*Fecha: 2026-06-13*
*Versión: 1.0*
