#!/usr/bin/env python3

import os
import re

def fix_rust_examples():
    # Find all rust example files
    examples_dir = "/workspaces/graph-sp/examples/rs"
    
    for filename in os.listdir(examples_dir):
        if filename.endswith('.rs'):
            filepath = os.path.join(examples_dir, filename)
            
            # Read file
            with open(filepath, 'r') as f:
                content = f.read()
            
            # Check if Arc is already imported
            if 'use std::sync::Arc;' not in content:
                # Add Arc import
                content = content.replace(
                    'use std::collections::HashMap;',
                    'use std::collections::HashMap;\nuse std::sync::Arc;'
                )
            
            # Replace graph.add calls with closures
            # Pattern: graph.add(\n        |...| {\n
            pattern = r'(\s+graph\.add\(\s*)\n(\s+)(\|[^|]*\|[^{]*\{)'
            
            def replacement(match):
                indent = match.group(2)
                return match.group(1) + '\n' + indent + 'Arc::new(' + match.group(3)
            
            new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE)
            
            # Now we need to close the Arc::new() calls
            # Find the matching closing brace and parenthesis for each function
            lines = new_content.split('\n')
            result_lines = []
            arc_new_depth = 0
            brace_depth = 0
            
            for i, line in enumerate(lines):
                result_lines.append(line)
                
                # Count Arc::new( occurrences
                arc_new_depth += line.count('Arc::new(|')
                
                if arc_new_depth > 0:
                    # Count braces
                    brace_depth += line.count('{') - line.count('}')
                    
                    # If we're back to balanced braces and we have a },
                    if brace_depth == 0 and line.strip().endswith('},'):
                        # Replace }, with }),
                        result_lines[-1] = line.replace('},', '}),')
                        arc_new_depth -= 1
            
            new_content = '\n'.join(result_lines)
            
            # Write back if changed
            if new_content != content:
                with open(filepath, 'w') as f:
                    f.write(new_content)
                print(f"Updated {filename}")

if __name__ == "__main__":
    fix_rust_examples()