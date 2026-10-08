#!/bin/bash

# Rust marker detection script for t27 corpus
# Detects specific Rust patterns found in the corpus

set -e

# Function to count distinct Rust markers in a file
count_rust_markers() {
    local file="$1"
    local markers=()
    
    # Check if file exists and is readable
    if [[ ! -f "$file" || ! -r "$file" ]]; then
        return 1
    fi
    
    # Read file content and search for markers
    local content
    content=$(< "$file")
    
    # Check for 'let' keyword (variable declarations)
    if grep -q '\blet\b' <<< "$content"; then
        markers+=("let")
    fi
    
    # Check for 'match' keyword
    if grep -q '\bmatch\b' <<< "$content"; then
        markers+=("match")
    fi
    
    # Check for 'for ... in' pattern
    if grep -q '\bfor\s+\w+\s+in\b' <<< "$content"; then
        markers+=("for_in")
    fi
    
    # Check for 'impl' blocks
    if grep -q '\bimpl\b' <<< "$content"; then
        markers+=("impl")
    fi
    
    # Check for fixed-size arrays [T; N]
    if grep -q '\[[^]]*;\s*[^]]*\]' <<< "$content"; then
        markers+=("array")
    fi
    
    # Check for Vec<> types
    if grep -q '\bVec\s*<' <<< "$content"; then
        markers+=("Vec")
    fi
    
    # Check for &self parameters
    if grep -q '\b&\s*self\b' <<< "$content"; then
        markers+=("&self")
    fi
    
    # Check for tuple return types (multiple types separated by commas)
    if grep -q '->\s*([^)]*,\s*[^)]*)' <<< "$content"; then
        markers+=("tuple_return")
    fi
    
    # Output the result
    local count=${#markers[@]}
    if [ "$count" -gt 0 ]; then
        echo "$file:$count"
        return 0
    else
        return 1
    fi
}

# Main function
main() {
    local search_dir="$1"
    
    # Default to specs/ if no directory provided
    if [[ -z "$search_dir" ]]; then
        search_dir="specs/"
    fi
    
    # Check if directory or file exists
    if [[ ! -e "$search_dir" ]]; then
        echo "Error: Path '$search_dir' not found" >&2
        exit 1
    fi
    
    # Find all .t27 files and process them
    local found_files=0
    local files_with_markers=0
    
    if [[ -f "$search_dir" ]]; then
        # Single file mode
        if count_rust_markers "$search_dir" >/dev/null; then
            count_rust_markers "$search_dir"
            files_with_markers=1
        fi
        found_files=1
    else
        # Directory mode - find all .t27 files
        while IFS= read -r -d '' file; do
            found_files=$((found_files + 1))
            if count_rust_markers "$file" >/dev/null; then
                count_rust_markers "$file"
                files_with_markers=$((files_with_markers + 1))
            fi
        done < <(find "$search_dir" -name "*.t27" -type f -print0)
    fi
    
    # Summary output (for debugging, not part of the requirement)
    if [[ "$#" -eq 0 || "$1" != "--quiet" ]]; then
        echo "Total .t27 files found: $found_files" >&2
        echo "Files with Rust markers: $files_with_markers" >&2
    fi
    
    # Output just the count for the wc -l test
    echo "$files_with_markers"
}

# Run main function with all arguments
main "$@"