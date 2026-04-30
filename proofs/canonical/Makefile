# Trinity Canonical Coq SSOT — make all → coqc verify 38 bundles · 376 Qed audited
# Anchor: phi^2 + phi^-2 = 3
# Census: github.com/gHashTag/trios/issues/373#issuecomment-4351659821

COQ_PROJECT := _CoqProject
COQ_MAKEFILE := Makefile.coq

all: $(COQ_MAKEFILE)
	$(MAKE) -f $(COQ_MAKEFILE) all

$(COQ_MAKEFILE): $(COQ_PROJECT)
	coq_makefile -f $(COQ_PROJECT) -o $(COQ_MAKEFILE)

clean: $(COQ_MAKEFILE)
	$(MAKE) -f $(COQ_MAKEFILE) cleanall
	$(RM) $(COQ_MAKEFILE) $(COQ_MAKEFILE).conf

verify-counts:
	@echo "=== Canonical Coq audit ==="
	@grep -rE '^(Theorem|Lemma|Corollary|Proposition|Example|Goal|Fact|Remark)\b' \
	      sacred/ kernel/ igla/ 2>/dev/null | wc -l | xargs -I{} echo "Theorem-like declarations: {}"
	@grep -rE '\bQed\.' sacred/ kernel/ igla/ 2>/dev/null | wc -l | xargs -I{} echo "Qed:         {}"
	@grep -rE '\bAdmitted\.' sacred/ kernel/ igla/ 2>/dev/null | wc -l | xargs -I{} echo "Admitted:    {}"
	@grep -rE '\bAbort\.' sacred/ kernel/ igla/ 2>/dev/null | wc -l | xargs -I{} echo "Abort/WIP:   {}"

.PHONY: all clean verify-counts
