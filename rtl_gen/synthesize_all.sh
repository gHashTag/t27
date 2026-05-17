#!/bin/bash
set -e

mkdir -p build reports

echo "=== Trinity t27 RTL Full Synthesis Report ==="
echo "Generated: $(date)"
echo ""

# List of modules to synthesize
modules="gf4_add gf8_add gf12_add gf16_add gf20_add gf24_add gf32_add gf64_add gf128_add gf256_add
          gf16_mul gf20_mul gf24_mul gf32_mul gf64_mul gf128_mul gf256_mul"

echo "Module,Cells,Wires,Bits" > reports/synthesis_summary.csv

for mod in $modules; do
    echo "Synthesizing $mod..."
    
    # Create yosys script
    cat > syn_${mod}.ys << YOSYS
# Read source files
read_verilog ${mod}.v
if [ -f gf_formats.v ]; then
    read_verilog gf_formats.v
fi

# Synthesis
hierarchy -check -top ${mod}
proc
opt_clean -purge
stat
write_json build/${mod}.json
write_verilog build/${mod}_synth.v
YOSYS

    # Run yosys
    if yosys syn_${mod}.ys > reports/${mod}.rpt 2>&1; then
        # Extract cell count
        cells=$(grep "Number of cells:" reports/${mod}.rpt | awk '{print $4}')
        wires=$(grep "Number of wires:" reports/${mod}.rpt | awk '{print $4}')
        bits=$(grep "Number of wire bits:" reports/${mod}.rpt | awk '{print $5}')
        
        echo "$mod,${cells:-0},${wires:-0},${bits:-0}" >> reports/synthesis_summary.csv
    else
        echo "$mod,ERROR,ERROR,ERROR" >> reports/synthesis_summary.csv
    fi
    
    rm -f syn_${mod}.ys
done

echo ""
echo "=== Synthesis Summary ==="
column -t -s, reports/synthesis_summary.csv
