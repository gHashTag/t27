#!/bin/bash
# Aggregate experience episodes and refresh brain seals
# Ring 059 - Experience aggregation pipeline

set -e

REPO_ROOT="${1:-.}"
EXPERIENCE_DIR="$REPO_ROOT/.trinity/experience"
SEALS_DIR="$REPO_ROOT/.trinity/seals"
BRAIN_SUMMARY="$SEALS_DIR/brain_summary.json"
BRAIN_DOMAINS="$SEALS_DIR/brain_domains.json"

echo "=== T27 Experience Aggregation Pipeline ==="
echo "Repo root: $REPO_ROOT"
echo ""

# Parse episodes
if [ -f "$EXPERIENCE_DIR/episodes.jsonl" ]; then
  TOTAL_EPISODES=$(wc -l < "$EXPERIENCE_DIR/episodes.jsonl")
  echo "Episodes: $TOTAL_EPISODES"
else
  echo "No episodes found at $EXPERIENCE_DIR/episodes.jsonl"
  TOTAL_EPISODES=0
fi

# Compute summary stats
CLEAN=0
TOXIC=0
WEAK_CONFIRM=0
LAST_RING=0

if [ "$TOTAL_EPISODES" -gt 0 ]; then
  while IFS= read -r line; do
    verdict=$(echo "$line" | jq -r '.verdict // "UNKNOWN"')
    ring=$(echo "$line" | jq -r '.ring_number // 0')

    case "$verdict" in
      "CLEAN") ((CLEAN++)) ;;
      "TOXIC") ((TOXIC++)) ;;
      "WEAK_CONFIRM") ((WEAK_CONFIRM++)) ;;
    esac

    if [ "$ring" -gt "$LAST_RING" ]; then
      LAST_RING=$ring
    fi
  done < "$EXPERIENCE_DIR/episodes.jsonl"
fi

echo "Verdicts: CLEAN=$CLEAN, TOXIC=$TOXIC, WEAK_CONFIRM=$WEAK_CONFIRM"
echo "Last Ring: $LAST_RING"
echo ""

# Generate brain_summary.json
cat > "$BRAIN_SUMMARY" << EOF
{
  "schema_version": 1,
  "seal_type": "brain_summary",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "ring_range": {
    "start": 0,
    "end": $LAST_RING
  },
  "summary": {
    "total_rings": $LAST_RING,
    "rings_closed": $CLEAN,
    "rings_open": $((LAST_RING - CLEAN)),
    "clean_verdicts": $CLEAN,
    "toxic_verdicts": $TOXIC,
    "weak_confirms": $WEAK_CONFIRM
  },
  "domains": {
    "seed_bootstrap": {"status": "SEALED", "health": 1.0, "last_ring": 31},
    "stem_conformance": {"status": "SEALED", "health": 1.0, "last_ring": 49},
    "branches_science": {"status": "SEALED", "health": 1.0, "last_ring": 58},
    "crown_automation": {"status": "ACTIVE", "health": 0.9, "last_ring": 59}
  },
  "queen_health": {
    "overall": "$([ $TOXIC -eq 0 ] && echo "GREEN" || echo "YELLOW")",
    "score": "$([ $LAST_RING -gt 0 ] && awk "BEGIN {printf \"%.4f\", $CLEAN / $LAST_RING}" || echo "1.0")",
    "domains": 4,
    "last_updated": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  }
}
EOF

echo "Generated: $BRAIN_SUMMARY"

# Generate brain_domains.json
cat > "$BRAIN_DOMAINS" << EOF
{
  "schema_version": 1,
  "seal_type": "brain_domains",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "domains": [
    {
      "name": "seed_bootstrap",
      "status": "SEALED",
      "health": 1.0,
      "last_ring": 31
    },
    {
      "name": "stem_conformance",
      "status": "SEALED",
      "health": 1.0,
      "last_ring": 49
    },
    {
      "name": "branches_science",
      "status": "SEALED",
      "health": 1.0,
      "last_ring": 58
    },
    {
      "name": "crown_automation",
      "status": "ACTIVE",
      "health": 0.9,
      "last_ring": 59
    },
    {
      "name": "experience_aggregation",
      "status": "ACTIVE",
      "health": 1.0,
      "last_ring": 59
    }
  ],
  "meta": {
    "total_domains": 5,
    "sealed_domains": 3,
    "active_domains": 2
  }
}
EOF

echo "Generated: $BRAIN_DOMAINS"
echo ""
echo "=== Brain seals refreshed ==="
