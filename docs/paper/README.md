# TAYNI Technical Paper

This folder contains the technical paper for TAYNI, suitable for submission to arXiv.

## Files

- `tayni-arxiv.tex` - Main LaTeX source
- `figures/` - Diagrams and charts (to be added)

## Building

```bash
# Requires LaTeX installation (texlive-full recommended)
pdflatex tayni-arxiv.tex
bibtex tayni-arxiv
pdflatex tayni-arxiv.tex
pdflatex tayni-arxiv.tex
```

Or use Overleaf for online compilation.

## arXiv Submission

1. Create account at https://arxiv.org
2. Choose category: `cs.PL` (Programming Languages)
3. Upload `.tex` file and any figures
4. Submit for moderation

## Paper Structure

1. **Abstract** - Summary of contributions
2. **Introduction** - Problem statement and motivation
3. **Design Principles** - Token efficiency, regular grammar, explicit semantics
4. **Language Syntax** - v1.0 and v1.5 variants
5. **Compiler Architecture** - Pipeline and IR design
6. **Implementation** - PE, ELF, Wasm generation details
7. **Evaluation** - Binary size and token benchmarks
8. **Related Work** - Comparison with Mojo, Triton, etc.
9. **Limitations and Future Work**
10. **Conclusion**

## Key Claims

- 64% token reduction vs Python
- 10.5KB HTTP server (vs 5.8MB Go)
- Zero external dependencies
- Capability-based security at compile time
- Multi-target: Windows, Linux, Wasm, WASI

## Citation

```bibtex
@article{tayni2026,
  title={TAYNI: An AI-First Programming Language for Token-Efficient Code Generation},
  author={NELAIA Project},
  journal={arXiv preprint},
  year={2026}
}
```
