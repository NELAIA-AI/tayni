import sys
import os

class NelaiaNativeOrchestrator:
    def __init__(self):
        self.memory = {}
        self.crates = {}
        self.artifact_buffer = []

    def load_stream(self, file_path):
        with open(file_path, 'r') as f:
            lines = [l.strip() for l in f.readlines() if l.strip() and not l.startswith('#')]
        
        for line in lines:
            tokens = self._tokenize(line)
            self._dispatch(tokens)

    def _tokenize(self, line):
        import shlex
        return shlex.split(line)

    def _dispatch(self, tokens):
        op = tokens[0]
        
        if op == "LOAD_CRATE":
            ref = tokens[1]
            crate = tokens[2]
            self.crates[ref] = crate
            print(f"[NELAIA-CORE] Native binding established: {crate} -> {ref}")

        elif op == "NATIVE_ALLOC":
            ref = tokens[1]
            type_enum = tokens[2]
            self.memory[ref] = {"type": type_enum, "value": None}
            print(f"[NELAIA-CORE] Zero-cost allocation: {ref} ({type_enum})")

        elif op == "STATIC_LOAD":
            ref = tokens[1]
            val = tokens[2]
            if ref in self.memory:
                self.memory[ref]["value"] = val
            print(f"[NELAIA-CORE] Memory populated: {ref} <- '{val}'")

        elif op == "DIRECT_CALL":
            crate_ref = tokens[1]
            func = tokens[2]
            arg_ref = tokens[3]
            
            crate = self.crates.get(crate_ref)
            arg_val = self.memory.get(arg_ref, {}).get("value")
            
            print(f"[NELAIA-CORE] Executing native jump: {crate}::{func}({arg_val})")
            
            # Simulate the native behavior that will be compiled
            if crate == "std.io" and func == "print":
                self.artifact_buffer.append(f'echo {arg_val}')
                
        elif op == "BUILD_STANDALONE":
            target = tokens[1]
            print(f"[NELAIA-CORE] Intercepting stream... Compiling standalone artifact for {target}")
            self._compile_artifact()

    def _compile_artifact(self):
        # We simulate compiling a native binary by creating a batch executable
        out_file = "hello_artifact.bat"
        with open(out_file, "w") as f:
            f.write("@echo off\n")
            for instruction in self.artifact_buffer:
                f.write(f"{instruction}\n")
            f.write("pause\n")
            
        print(f"\n[SUCCESS] Standalone artifact generated: {out_file}")
        print("          This artifact has ZERO dependencies on NELAIA.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python orchestrator.py <stream.nela>")
        sys.exit(1)
        
    core = NelaiaNativeOrchestrator()
    core.load_stream(sys.argv[1])
