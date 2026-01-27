import sys

# Read main.rs
with open(r'C:\Users\frank\Documents\scripts\trading-scripts\ladybug\rust-engine\src\main.rs', 'r', encoding='utf-8') as f:
    lines = f.readlines()

# Read new function
with open(r'C:\Users\frank\Documents\scripts\trading-scripts\ladybug\new_generate_test_data.txt', 'r', encoding='utf-8') as f:
    new_function = f.read()

# Find start and end (lines are 0-indexed, but line numbers are 1-indexed)
start_line = 799  # Line 800 in editor (async fn generate_test_data)
end_line = 963    # Line 964 in editor (closing brace before clear_test_data)

# Replace
new_lines = lines[:start_line] + [new_function + '\n'] + lines[end_line:]

# Write back
with open(r'C:\Users\frank\Documents\scripts\trading-scripts\ladybug\rust-engine\src\main.rs', 'w', encoding='utf-8') as f:
    f.writelines(new_lines)

print("SUCCESS: Replaced generate_test_data function")
print(f"Old function: lines {start_line+1}-{end_line+1}")
print(f"New function: {len(new_function.split(chr(10)))} lines")
