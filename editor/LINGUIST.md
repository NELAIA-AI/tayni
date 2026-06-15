# NELAIA Language Registration for GitHub Linguist

This document describes the NELAIA programming language for registration in GitHub Linguist.

## Language Information

| Property | Value |
|----------|-------|
| Name | NELAIA |
| Type | Programming |
| Extension | `.nela` |
| Color | `#4A90D9` |
| Ace Mode | text |
| TM Scope | source.nela |

## Description

NELAIA is a graph-based metalanguage designed for AI code generation. It compiles to native executables (Windows PE, Linux ELF) without external dependencies.

## Key Features

- Graph-based paradigm (nodes and data flow)
- Token-efficient syntax
- Direct native code emission
- Capability-based security model
- Designed for AI agents to generate

## Sample Code

```nela
-- Hello World
.msg: "Hello from NELAIA!\n"
.len: 20
.out: PRT .msg .len
```

```nela
-- HTTP Server
.caps: REQUIRES { http }
.port: 8080
.server: HTTP.LISTEN .port
.client: HTTP.ACCEPT .server
.response: "Hello!"
.send: HTTP.RESPOND .client 200 .response
```

```nela
-- Fibonacci
.a: 0
.b: 1
.n: 10
.loop: BRN .n .done
  .temp: .b
  .b: ADD .a .b
  .a: .temp
  .n: SUB .n 1
  > .loop
.done: PRT .b
```

## Grammar (EBNF)

```ebnf
program     = { node | flow | comment } ;
node        = "." identifier ":" ( literal | operation ) ;
literal     = number | string ;
operation   = operator { argument } ;
argument    = "." identifier | literal ;
flow        = "." identifier ">" "." identifier ;
comment     = "--" { character } newline ;
identifier  = letter { letter | digit | "_" } ;
```

## Repository

- Main: https://github.com/NELAIA-AI/nelaia-core
- Documentation: https://github.com/NELAIA-AI/nelaia-core/tree/main/docs

## TextMate Grammar

Available at: `editor/vscode/nelaia.tmLanguage.json`

## Linguist Entry (proposed)

```yaml
NELAIA:
  type: programming
  color: "#4A90D9"
  extensions:
    - ".nela"
  tm_scope: source.nela
  ace_mode: text
  language_id: 1000001  # To be assigned
```

## Notes

- The language is actively developed
- Compiler available for Windows and Linux
- Designed primarily for AI code generation
- Human-readable but optimized for machine generation
