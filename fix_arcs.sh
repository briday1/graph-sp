#!/bin/bash
# Fix all remaining Arc::new issues in Rust files

# Find all files with closure patterns that need Arc::new
for file in examples/rs/*.rs tests/*.rs; do
  if [[ -f "$file" ]]; then
    echo "Fixing $file"
    
    # Fix simple function names in .add()
    sed -i 's/\.add(\s*\([a-z_][a-z_0-9]*\),/\.add(Arc::new(\1),/g' "$file"
    
    # Fix closures - add Arc::new before |inputs|
    sed -i 's/\.add(\s*|/\.add(Arc::new(|/g' "$file"
    
    # Fix missing closing parens after closures
    # This is tricky - we need to find the matching }
    python3 -c "
import re
import sys

def fix_closures(content):
    lines = content.split('\n')
    result = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check if this is an Arc::new(| line that needs a closing paren
        if 'Arc::new(|' in line and line.strip().endswith('{'):
            result.append(line)
            i += 1
            
            # Count braces to find the end
            brace_count = 1
            while i < len(lines) and brace_count > 0:
                current_line = lines[i]
                brace_count += current_line.count('{') - current_line.count('}')
                
                if brace_count == 0:
                    # Add closing paren after the }
                    if current_line.strip().endswith('},'):
                        result.append(current_line.replace('},', '}),'))
                    elif current_line.strip().endswith('}'):
                        result.append(current_line + ')')
                    else:
                        # Find the } and add paren after it
                        result.append(re.sub(r'}(\s*,?\s*)$', r'})\1', current_line))
                else:
                    result.append(current_line)
                i += 1
        else:
            result.append(line)
            i += 1
    
    return '\n'.join(result)

with open('$file', 'r') as f:
    content = f.read()

content = fix_closures(content)

with open('$file', 'w') as f:
    f.write(content)
"
  fi
done