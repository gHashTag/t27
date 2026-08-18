"""MathML -> styled HTML. WeasyPrint does not render MathML; it flattens every
child text node, so the presentation markup AND pandoc's <annotation> LaTeX both
land on the page and every formula prints twice. This walks the subset the
article actually uses and emits <sup>/<sub>/fraction spans instead."""
import re, sys, html
from xml.etree import ElementTree as ET

MJ = "{http://www.w3.org/1998/Math/MathML}"
def tag(e): return e.tag.replace(MJ, "")

def render(e):
    t = tag(e)
    if t == "annotation" or t == "annotation-xml":
        return ""
    if t in ("mi", "mn", "mo", "mtext", "ms"):
        s = html.escape((e.text or "").strip())
        if t == "mi" and len(s) == 1 and s.isalpha():
            return f'<i class="mi">{s}</i>'
        if t == "mo":
            return f'<span class="mo">{s}</span>' if s else ""
        return s
    kids = [render(c) for c in e]
    kids = [k for k in kids if k]
    if t == "msup":
        return f'{kids[0]}<sup>{"".join(kids[1:])}</sup>' if len(kids) >= 2 else "".join(kids)
    if t == "msub":
        return f'{kids[0]}<sub>{"".join(kids[1:])}</sub>' if len(kids) >= 2 else "".join(kids)
    if t == "msubsup":
        return f'{kids[0]}<sub>{kids[1]}</sub><sup>{"".join(kids[2:])}</sup>' if len(kids) >= 3 else "".join(kids)
    if t == "mfrac" and len(kids) >= 2:
        return f'<span class="frac"><span class="num">{kids[0]}</span><span class="den">{kids[1]}</span></span>'
    if t == "msqrt":
        return f'<span class="sqrt">&radic;<span class="rad">{"".join(kids)}</span></span>'
    if t in ("semantics", "mrow", "mstyle", "mpadded", "math"):
        return "".join(kids)
    return "".join(kids)

def convert(doc):
    out, pos = [], 0
    for m in re.finditer(r'<math\b.*?</math>', doc, re.S):
        out.append(doc[pos:m.start()])
        frag = m.group(0)
        try:
            root = ET.fromstring(frag)
            inner = render(root)
            block = 'display="block"' in frag
            cls = "mathblock" if block else "math"
            out.append(f'<span class="{cls}">{inner}</span>')
        except Exception:
            # never emit the raw LaTeX fallback; drop to the flattened text
            txt = re.sub(r'<annotation[^>]*>.*?</annotation>', '', frag, flags=re.S)
            out.append(re.sub(r'<[^>]+>', '', txt))
        pos = m.end()
    out.append(doc[pos:])
    return "".join(out)

if __name__ == "__main__":
    src = open(sys.argv[1]).read()
    n = len(re.findall(r'<math\b', src))
    open(sys.argv[2], "w").write(convert(src))
    print(f"  converted {n} MathML elements to typographic HTML")
