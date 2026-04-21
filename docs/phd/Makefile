# PhD Monograph Makefile
# Trinity Identity Framework

.PHONY: all clean coq figures verify status

# Main target
all: monograph.pdf

# Build monograph PDF
monograph.pdf: main.tex bibliography.bib
	@echo "Building monograph..."
	pdflatex -interaction=nonstopmode main.tex > build.log 2>&1
	@if [ -f monograph.pdf ]; then \
		pdfinfo monograph.pdf | grep "Pages:"; \
	else \
		echo "PDF build failed - check build.log"; \
		exit 1; \
	fi
	@echo "Monograph build complete."

# Run bibliography processing
bibliography: bibliography.bib
	@echo "Bibliography ready: $(grep -c '^@' bibliography.bib) entries"

# Coq compilation target
coq:
	@if [ -d coq ]; then \
		echo "Compiling Coq files..."; \
		cd coq && for f in *.v; do \
			if [ -f "$$f" ]; then \
				echo "  Compiling $$f..."; \
				coqc -R . "$$f" 2>&1 | head -5; \
			fi; \
		done; \
		./scripts/verify-coq.sh; \
	else \
		echo "Coq directory not found"; \
	fi

# Figures target (placeholder for external figure generation)
figures:
	@if [ -d figures ]; then \
		echo "Figures directory exists with $$(find figures -type f | wc -l) files"; \
	else \
		echo "Figures directory not found - creating..."; \
		mkdir -p figures; \
	fi

# Run all verification scripts
verify:
	@echo "Running verification..."
	@./scripts/verify-all.sh

# Quick status check
status:
	@echo "=== Monograph Status ==="
	@if [ -f monograph.pdf ]; then \
		pdfinfo monograph.pdf | grep "Pages:"; \
	else \
		echo "PDF: NOT BUILT"; \
	fi
	@echo ""
	@echo "=== Content Status ==="
	@echo "Chapters: $$(find chapters -name "*.tex" 2>/dev/null | wc -l) files"
	@echo "Total lines: $$(find chapters -name "*.tex" -exec wc -l {} + 2>/dev/null | awk '{s+=$$1} END {print s}')"
	@echo ""
	@echo "=== Bibliography ==="
	@echo "Entries: $$(grep -c '^@' bibliography.bib 2>/dev/null || echo 0)"
	@echo ""
	@echo "=== Coq Proofs ==="
	@echo ".v files: $$(find coq -name "*.v" 2>/dev/null | wc -l | tr -d ' ' || echo 0)"
	@echo ".vo files: $$(find coq -name "*.vo" 2>/dev/null | wc -l | tr -d ' ' || echo 0)"
	@echo ""
	@echo "=== Figures ==="
	@if [ -d figures ]; then \
		find figures -type f | wc -l; \
	else \
		echo "Directory not found"; \
	fi

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -f *.aux *.log *.out *.toc *.lof *.lot *.bbl *.blg *.synctex.gz
	rm -f main.pdf
	@if [ -d coq ]; then \
		cd coq && rm -f *.vo *.glob; \
	fi
	@echo "Clean complete."

# Deep clean (remove PDF)
distclean: clean
	@echo "Removing all generated files..."
	rm -f monograph.pdf
	@echo "Distclean complete."
