#!/bin/bash
# OSF Registration Script for Trinity γ-Hypotheses Preregistration (v0.2)
# Date: 2026-04-08
# Token: OAuth2 Bearer token

set -e

# Configuration
OSF_API="https://api.osf.io/v2"
TOKEN="YTAWGw0CniqqGRJAARJIky6lqhNSwYOpMav8C8CLG2mOT9zlZrw5DsrEGY1MiNHZDpbXTV"
REPO_ROOT="/Users/playra/t27"

echo "=========================================="
echo "OSF API Registration: Trinity γ-Hypotheses"
echo "=========================================="
echo ""

# Files to upload
SEAL_FILE="$REPO_ROOT/research/seals/smoking_guns_v1.sha"
PREREG_FILE="$REPO_ROOT/research/gamma-hypotheses/OSF-preregistration.md"
FORMULA_FILE="$REPO_ROOT/research/trinity-pellis-paper/FORMULA_TABLE.md"

# Verify files exist
for file in "$SEAL_FILE" "$PREREG_FILE" "$FORMULA_FILE"; do
    if [ ! -f "$file" ]; then
        echo "ERROR: File not found: $file"
        exit 1
    fi
done

# Step 1: Create Registration
echo "[Step 1] Creating OSF registration..."
RESPONSE=$(curl -s -X POST "$OSF_API/registrations/" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/vnd.api+json" \
  -d '{
    "title": "Trinity γ-Hypotheses Preregistration (v0.2)",
    "description": "Preregistration of Trinity conjecture GI1: γ = φ⁻³ = √5−2. Conjecture proposes Barbero-Immirzi parameter derived from golden ratio φ. Gap to Meissner (2004) γ₁ is only 0.62%, substantially smaller than internal LQG dispute (13.9%). Evidence includes exact closed form, Domagala-Lewandowski bounds satisfaction, and 14 SMOKING GUN formulas with Δ < 0.1% verified at 50-digit precision.",
    "category": "Physics",
    "tags": ["Loop Quantum Gravity", "Immirzi Parameter", "Golden Ratio", "Trinity S³AI", "Barbero-Immirzi"]
  }')

if [ $? -ne 0 ]; then
    echo "ERROR: Registration creation failed"
    echo "$RESPONSE"
    exit 1
fi

# Extract registration ID
REGISTRATION_ID=$(echo "$RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")

if [ -z "$REGISTRATION_ID" ]; then
    echo "ERROR: Could not extract registration ID from response"
    echo "$RESPONSE"
    exit 1
fi

echo "Registration ID: $REGISTRATION_ID"
echo "Registration URL: https://osf.io/registrations/$REGISTRATION_ID/"
echo ""

# Step 2: Upload Files
echo "[Step 2] Uploading files..."

# Function to upload file
upload_file() {
    local file_path="$1"
    local file_name=$(basename "$file_path")
    local upload_path="$2"
    local file_content=$(cat "$file_path")

    echo "  Uploading: $file_name to $upload_path"

    UPLOAD_RESPONSE=$(curl -s -X POST "$OSF_API/nodes/$REGISTRATION_ID/files/" \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/vnd.api+json" \
      -d "{
        \"name\": \"$file_name\",
        \"kind\": \"file\",
        \"path\": \"$upload_path\",
        \"content\": \"$file_content\"
      }")

    if [ $? -ne 0 ]; then
        echo "  ERROR: Failed to upload $file_name"
        echo "  $UPLOAD_RESPONSE"
        return 1
    fi

    # Extract file ID for verification
    FILE_ID=$(echo "$UPLOAD_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('id', ''))")
    echo "  File ID: $FILE_ID"
}

# Upload each file
upload_file "$SEAL_FILE" "/seals/"
upload_file "$PREREG_FILE" "/"
upload_file "$FORMULA_FILE" "/trinity-pellis-paper/"

echo ""

# Step 3: Retrieve DOI
echo "[Step 3] Retrieving DOI..."
DOI_RESPONSE=$(curl -s "$OSF_API/registrations/$REGISTRATION_ID/")

if [ $? -ne 0 ]; then
    echo "ERROR: Failed to retrieve DOI"
    exit 1
fi

# Extract DOI
DOI=$(echo "$DOI_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin).get('doi', '').get('url', 'N/A'))")

echo "DOI: $DOI"
echo ""

# Step 4: Update Paper Draft
echo "[Step 4] Updating paper draft..."

if [ "$DOI" = "N/A" ] || [ -z "$DOI" ]; then
    echo "WARNING: DOI not available, cannot update draft"
    echo "Registration URL: https://osf.io/registrations/$REGISTRATION_ID/"
else
    # Update the draft file
    sed -i '' "s/\*\*DOI:\*\* TBD/\*\*DOI:\*\* $DOI/" "$REPO_ROOT/research/trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.2.md"
    echo "Updated: $REPO_ROOT/research/trinity-gamma-paper/GAMMA_PAPER_DRAFT_v0.2.md"
fi

echo ""
echo "=========================================="
echo "Registration Complete!"
echo "=========================================="
echo "Registration URL: https://osf.io/registrations/$REGISTRATION_ID/"
echo "DOI: $DOI"
echo ""
echo "Next steps:"
echo "1. Verify files are visible at registration URL"
echo "2. Submit to arXiv: https://arxiv.org/submit (category: gr-qc)"
echo "3. Update repository with DOI link"
