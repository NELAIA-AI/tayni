# TAYNI Bootstrap

Este directorio contiene los binarios necesarios para compilar TAYNI desde cero.

## Problema del Huevo y la Gallina

TAYNI es un lenguaje self-hosting: el compilador está escrito en TAYNI. Para compilar TAYNI necesitas un compilador TAYNI. Este es el clásico problema de bootstrap que todos los lenguajes self-hosting enfrentan (Rust, Go, GCC, etc.).

## Solución

Proporcionamos binarios pre-compilados para Windows x64:

| Archivo | Descripción |
|---------|-------------|
| `tayni-bootstrap.exe` | Compilador TAYNI mínimo |
| `mini.exe` | Template PE mínimo |
| `fileio2.exe.exe` | Template PE con syscalls |

## Uso

```bash
# 1. Crear input.tyn con código TAYNI (valor 0-9)
echo .x: 5 > input.tyn

# 2. Compilar (compiler.exe debe existir)
tayni-bootstrap.exe

# 3. Ejecutar el resultado
out.exe
echo %ERRORLEVEL%  # Muestra: 5

# 4. SELF-REPLICATION: out.exe puede compilar también
copy out.exe compiler.exe
echo .x: 7 > input.tyn
compiler.exe
# Genera nuevo out.exe con valor 7
```

## Self-Replication

El bootstrap compiler es **self-replicating**:
- `out.exe` generado es idéntico a `compiler.exe`
- Cada generación puede generar más compiladores
- Cadena infinita: compiler.exe → out.exe → out.exe → ...

## Limitaciones del Bootstrap

El bootstrap compiler es mínimo:
- Solo soporta formato `.x: N` donde N es un dígito (0-9)
- Genera un exe que retorna el valor como exit code
- Requiere `compiler.exe` en el mismo directorio

Para funcionalidad completa, usa el compilador principal.

## Verificación

```bash
# El bootstrap compiler debe poder compilar código simple
echo .x: 42 > input.tyn
tayni-bootstrap.exe
out.exe
# Exit code debe ser 42
```

## Próximos Pasos

Una vez que tengas el bootstrap funcionando:
1. Compila el compilador completo desde `src/tayni/`
2. Usa el compilador completo para desarrollo
3. El bootstrap solo se necesita una vez

## Plataformas

- **Windows x64**: ✅ Incluido
- **Linux x64**: 🔜 Próximamente
- **macOS**: 🔜 Próximamente

## Nota sobre Dependencias

El bootstrap compiler requiere:
- Windows 10/11 x64
- No requiere runtime adicional
- No requiere instalación

Los binarios son completamente autónomos.
