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

# 2. Compilar
tayni-bootstrap.exe

# 3. Ejecutar
out.exe
echo %ERRORLEVEL%  # Muestra: 5
```

## Limitaciones del Bootstrap

El bootstrap compiler es mínimo:
- Solo soporta formato `.x: N` donde N es un dígito (0-9)
- Genera un exe que retorna el valor como exit code
- No soporta operaciones complejas

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
