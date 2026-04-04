#!/bin/bash
# swarm-health.sh - Aggregate Queen Trinity health and swarm status
#
# Usage: ./swarm-health.sh [--full|--domain <domain>|--watch]
#
# Aggregates: queen_health, toxic rate, repeated failures,
# last clean seal, blocked graph edges, recovery status

set -euo pipefail

MODE="${1:--full}"
WATCH_INTERVAL=5

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Health domains with their file indicators
declare -A DOMAIN_FILES=(
    ["sacredphysics"]="specs/math/sacred_physics.t27 conformance/sacred-physics.json"
    ["numeric"]="specs/numeric/gf*.t27 conformance/gf*-conformance.json"
    ["graph"]="architecture/graph.tri architecture/graphv2.json"
    ["compiler"]="compiler/**/*.t27"
    ["runtime"]="specs/runtime/**/*.t27"
    ["queenlotus"]=".claude/skills/tri/SKILL.md"
)

# Calculate domain health (0.0 - 1.0)
calc_domain_health() {
    local domain="$1"
    local health=1.0

    # Check if domain files exist
    for file in ${DOMAIN_FILES[$domain]}; do
        if [[ ! -f "$file" ]] && [[ ! "$file" == *"*"* ]]; then
            health=0.0
            break
        fi
    done

    # Check for toxic verdicts in decision log
    if [[ -f ".claude/skills/tri/decision-log.jsonl" ]]; then
        local toxic_count=$(grep -c "\"verdict\":\"toxic\"" ".claude/skills/tri/decision-log.jsonl" 2>/dev/null || echo "0")
        local total_count=$(wc -l < ".claude/skills/tri/decision-log.jsonl" 2>/dev/null || echo "1")
        local toxic_rate=$(echo "scale=3; $toxic_count / $total_count" | bc 2>/dev/null || echo "0")

        # Reduce health if toxic rate > 20%
        if (( $(echo "$toxic_rate > 0.2" | bc -l 2>/dev/null || echo "0") )); then
            health=0.3
        elif (( $(echo "$toxic_rate > 0.05" | bc -l 2>/dev/null || echo "0") )); then
            health=0.7
        fi
    fi

    echo "$health"
}

# Calculate queen health (weighted aggregate)
calc_queen_health() {
    local sacredphysics=$(calc_domain_health "sacredphysics")
    local numeric=$(calc_domain_health "numeric")
    local graph=$(calc_domain_health "graph")
    local compiler=$(calc_domain_health "compiler")
    local runtime=$(calc_domain_health "runtime")
    local queenlotus=$(calc_domain_health "queenlotus")

    # Queen health formula from queen-health.md
    local queen_health=$(echo "scale=3;
        $sacredphysics * 0.30 +
        $numeric * 0.25 +
        $graph * 0.20 +
        $compiler * 0.10 +
        $runtime * 0.10 +
        $queenlotus * 0.05" | bc)

    echo "$queen_health"
}

# Get health status color
get_status_color() {
    local health="$1"
    if (( $(echo "$health < 0.5" | bc -l) )); then
        echo -n "$RED"
    elif (( $(echo "$health < 0.7" | bc -l) )); then
        echo -n "$YELLOW"
    else
        echo -n "$GREEN"
    fi
}

# Get health status label
get_status_label() {
    local health="$1"
    if (( $(echo "$health < 0.5" | bc -l) )); then
        echo -n "RED"
    elif (( $(echo "$health < 0.7" | bc -l) )); then
        echo -n "YELLOW"
    else
        echo -n "GREEN"
    fi
}

# Show toxic verdict statistics
show_toxic_stats() {
    if [[ -f ".claude/skills/tri/decision-log.jsonl" ]]; then
        echo -e "\n📊 Toxic Verdict Statistics:"

        local toxic_count=$(grep -c "\"verdict\":\"toxic\"" ".claude/skills/tri/decision-log.jsonl" 2>/dev/null || echo "0")
        local total_count=$(wc -l < ".claude/skills/tri/decision-log.jsonl" 2>/dev/null || echo "1")
        local toxic_rate=$(echo "scale=1; $toxic_count * 100 / $total_count" | bc 2>/dev/null || echo "0")

        echo "  Toxic Rate: ${toxic_rate}% (${toxic_count}/${total_count})"

        # Find repeated failures
        echo -e "\n🔄 Repeated Failures:"
        grep "\"verdict\":\"toxic\"" ".claude/skills/tri/decision-log.jsonl" 2>/dev/null | \
            grep -o '"spec_path":"[^"]*"' | sort | uniq -c | sort -rn | \
            awk '$1 > 1 { print "    " $2 ": " $1 " times" }' || \
            echo "    No repeated failures"
    fi
}

# Show last clean seal
show_last_clean() {
    echo -e "\n🔒 Last Clean Seal:"
    if [[ -f ".claude/skills/tri/seals.jsonl" ]]; then
        grep '"verdict":"clean"' ".claude/skills/tri/seals.jsonl" 2>/dev/null | tail -1 | \
            jq -r '"  Seal: \(.seal_id)\n  Spec: \(.spec_path)\n  At: \(.sealed_at)"' 2>/dev/null || \
            echo "    No clean seals found"
    else
        echo "    No seals recorded"
    fi
}

# Show blocked graph edges
show_blocked_edges() {
    echo -e "\n🚫 Blocked Graph Edges:"
    if [[ -f "architecture/graph.tri" ]]; then
        grep -n "blocked\|toxic\|error" architecture/graph.tri 2>/dev/null | \
            sed 's/^/    /' || echo "    No blocked edges"
    else
        echo "    Graph file not found"
    fi
}

# Show full health report
show_full_report() {
    local queen_health=$(calc_queen_health)
    local status_color=$(get_status_color "$queen_health")
    local status_label=$(get_status_label "$queen_health")

    echo -e "\n╔══════════════════════════════════════════════════════════════╗"
    echo -e "║         Queen Trinity Health Report                        ║"
    echo -e "╚══════════════════════════════════════════════════════════════╝"

    echo -e "\n👑 Queen Health: ${status_color}${queen_health}${NC} (${status_label})"

    echo -e "\n📋 Domain Health:"
    for domain in sacredphysics numeric graph compiler runtime queenlotus; do
        local health=$(calc_domain_health "$domain")
        local color=$(get_status_color "$health")
        local label=$(get_status_label "$health")
        printf "  %-15s ${color}%.2f${NC} (%s)\n" "${domain^}" "$health" "$label"
    done

    show_toxic_stats
    show_last_clean
    show_blocked_edges

    # Operating recommendations
    echo -e "\n⚡ Operating Status:"
    if (( $(echo "$queen_health < 0.5" | bc -l) )); then
        echo -e "  ${RED}CRITICAL${NC} — Only recovery loops allowed"
        echo "  → Run: tri replay-step --last-clean"
    elif (( $(echo "$queen_health < 0.7" | bc -l) )); then
        echo -e "  ${YELLOW}WARNING${NC}  — Feature mutations blocked on affected domains"
        echo "  → Run: tri recovery domain --affected <domain>"
    else
        echo -e "  ${GREEN}HEALTHY${NC}  — All operations allowed"
    fi

    echo ""
}

# Show single domain health
show_domain_health() {
    local domain="$1"
    local health=$(calc_domain_health "$domain")
    local color=$(get_status_color "$health")
    local label=$(get_status_label "$label")

    echo "${domain^} Health: ${color}${health}${NC} (${label})"
}

# Watch mode
watch_mode() {
    while true; do
        clear
        show_full_report
        echo -e "\nRefreshing every ${WATCH_INTERVAL}s... (Ctrl+C to exit)"
        sleep "$WATCH_INTERVAL"
    done
}

# Main execution
case "$MODE" in
    --full)
        show_full_report
        ;;
    --domain)
        show_domain_health "$2"
        ;;
    --watch)
        watch_mode
        ;;
    --json)
        # JSON output for programmatic consumption
        local queen_health=$(calc_queen_health)
        cat <<EOF
{
  "queen_health": $queen_health,
  "status": "$(get_status_label "$queen_health")",
  "domains": {
    "sacredphysics": $(calc_domain_health "sacredphysics"),
    "numeric": $(calc_domain_health "numeric"),
    "graph": $(calc_domain_health "graph"),
    "compiler": $(calc_domain_health "compiler"),
    "runtime": $(calc_domain_health "runtime"),
    "queenlotus": $(calc_domain_health "queenlotus")
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
        ;;
    *)
        echo "Usage: $0 [--full|--domain <domain|--watch|--json]" >&2
        exit 1
        ;;
esac
