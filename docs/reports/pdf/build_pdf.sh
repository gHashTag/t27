set -e
S=/private/tmp/claude-501/-Users-playom-t27--claude-worktrees-igla-fpga-improvements-3f5e1a/eeed4a0e-20e8-40f4-aa16-1ecfee4ad92d/scratchpad/w846
cd /Users/playom/t27/.claude/worktrees/igla-fpga-improvements-3f5e1a
SRC=${1:-docs/theory/TNF_ARTICLE_RU.md}
OUT=${2:-$S/TNF.pdf}
pandoc "$SRC" -f markdown -t html5 --mathml --standalone \
       --metadata title="Ternary Network Floats" -o $S/body.html
python3 - "$S" <<'PY'
import sys, datetime, subprocess, pathlib
S = sys.argv[1]
h = pathlib.Path(f"{S}/body.html").read_text()
rev = subprocess.run(["git","rev-parse","--short","HEAD"],capture_output=True,text=True).stdout.strip()
title = ("<div class=\"titlepage\"><div class=\"rule\"></div>"
  "<h1>Ternary Network Floats</h1>"
  "<div class=\"sub\">Троичные сетевые числа с плавающей точкой:<br>формат, кремний и методология измерения</div>"
  f"<div class=\"meta\"><b>Проект t27 · Trinity S&sup3;AI</b><br>"
  f"ревизия <b>{rev}</b> · собрано {datetime.date.today().isoformat()}<br>"
  "источник <b>docs/theory/TNF_ARTICLE_RU.md</b></div>"
  "<div class=\"standing\"><b>Действующее правило измерения (T619a).</b> Кремниевый вердикт "
  "обязан совпасть минимум на трёх сидах размещения, иначе он является утверждением об "
  "одном размещении, а не о спецификации. Основание: на неизменном нетлисте достигнутая "
  "частота прошла от 15.83 до 18.29 МГц, а три размещения из пяти вычисляли заданную "
  "функцию и два — нет, детерминированно.</div></div>")
pathlib.Path(f"{S}/paper.html").write_text(h.replace("<body>", "<body>"+title, 1))
PY
python3 $S/mathml2html.py $S/paper.html $S/paper2.html
$S/venv/bin/python -c "
import sys
from weasyprint import HTML, CSS
HTML('$S/paper2.html').write_pdf(sys.argv[1], stylesheets=[CSS('$S/paper.css')])
" "$OUT"
$S/venv/bin/python -c "
from pypdf import PdfReader
import sys; print('  pages:', len(PdfReader(sys.argv[1]).pages))
" "$OUT"
