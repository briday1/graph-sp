#!/usr/bin/env python3

import re
import os

def add_arc_import(content):
    """Add Arc import if not present."""
    if 'use std::sync::Arc;' not in content:
        lines = content.split('\n')
        
        # Find a good place to add the import
        insert_pos = 0
        for i, line in enumerate(lines):
            if line.strip().startswith('use '):
                insert_pos = i + 1
        
        # Insert the Arc import
        lines.insert(insert_pos, 'use std::sync::Arc;')
        content = '\n'.join(lines)
    
    return content

def fix_simple_function_calls(content):
    """Fix simple function names passed to .add()."""
    
    # Pattern: graph.add(function_name, ...)
    content = re.sub(
        r'(\b(?:graph|branch_[abc]|branch|stats_branch|ml_branch)\.add\(\s*)([a-z_][a-z_0-9]*)(,)',
        r'\1Arc::new(\2)\3',
        content
    )
    
    return content

def fix_simple_closures(content):
    """Fix simple inline closures."""
    
    lines = content.split('\n')
    result = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Look for .add( followed by simple closure
        if re.search(r'\.(add)\(\s*$', line):
            result.append(line)
            i += 1
            
            # Check if next line is a simple closure
            if i < len(lines) and re.match(r'^\s*\|.*\|\s*\{', lines[i]):
                # Wrap the closure with Arc::new
                closure_line = lines[i]
                indentation = re.match(r'^(\s*)', closure_line).group(1)
                wrapped_start = re.sub(r'^(\s*)', r'\1Arc::new(', closure_line, 1)
                result.append(wrapped_start)
                i += 1
                
                # Find the closing brace and add closing paren
                brace_count = 1
                while i < len(lines) and brace_count > 0:
                    current_line = lines[i]
                    brace_count += current_line.count('{') - current_line.count('}')
                    
                    if brace_count == 0:
                        # Add closing paren
                        if current_line.strip().endswith('},'):
                            result.append(current_line.replace('},', '}),'))
                        else:
                            result.append(current_line + ')')
                        i += 1
                        break
                    else:
                        result.append(current_line)
                        i += 1
            else:
                continue
        else:
            result.append(line)
            i += 1
    
    return '\n'.join(result)

def main():
    """Fix the most critical compilation errors first."""
    
    # Start with the integration tests since those are most important
    test_file = '/workspaces/graph-sp/tests/integration_tests.rs'
    
    print(f"Fixing {test_file}")
    
    with open(test_file, 'r') as f:
        content = f.read()
    
    content = add_arc_import(content)
    content = fix_simple_function_calls(content)
    content = fix_simple_closures(content)
    
    with open(test_file, 'w') as f:
        f.write(content)
    
    print("Done with integration tests")

if __name__ == '__main__':
    main()