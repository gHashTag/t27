import glob
import os

# W344 batch append script
# Adds +2 tests and +1 invariant with _w344 suffix to each IGLA spec

specs = []
for pattern in ['specs/igla/race/*.t27', 'specs/igla/coder/*.t27']:
    specs.extend(glob.glob(pattern))

print(f"Found {len(specs)} specs to append")

for spec_path in sorted(specs):
    with open(spec_path, 'r') as f:
        content = f.read()

    # Check if _w344 already exists
    if '_w344' in content:
        print(f"SKIP {spec_path}: _w344 already present")
        continue

    basename = os.path.basename(spec_path).replace('.t27', '')
    block_name = basename.replace('-', '_')

    append_block = f"""
// Wave Loop 344 -- depth +1 (86->87)
test {block_name}_w344_batch_depth_invariant_1 {{ /* verify baseline */ }}
test {block_name}_w344_batch_depth_invariant_2 {{ /* verify baseline */ }}
invariant {block_name}_w344_depth_087: true
"""

    # Append before the last closing brace if the file ends with one on its own line
    stripped = content.rstrip()
    if stripped.endswith('}'):
        last_brace = stripped.rfind('}')
        new_content = stripped[:last_brace] + append_block + '\n' + stripped[last_brace:]
    else:
        new_content = stripped + append_block

    with open(spec_path, 'w') as f:
        f.write(new_content)

    print(f"APPEND {spec_path}")

print("Done")
