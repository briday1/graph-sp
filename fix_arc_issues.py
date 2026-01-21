#!/usr/bin/env python3
"""
Script to fix all Arc::new() wrapping issues in Rust examples and tests.
"""

import os
import re
import glob

def fix_function_calls(content):
    """Fix function calls that need Arc::new wrapping."""
    
    # Pattern 1: Simple function names passed to .add()
    # Match: graph.add(function_name, ...)
    content = re.sub(
        r'(\b(?:graph|branch_[abc]|branch|stats_branch|ml_branch)\.add\(\s*)([a-z_][a-z_0-9]*)(,)',
        r'\1Arc::new(\2)\3',
        content
    )
    
    # Pattern 2: Closures in .add() - single line
    content = re.sub(
        r'(\b(?:graph|branch_[abc]|branch|stats_branch|ml_branch)\.add\(\s*)(\|[^|]*\|[^{]*\{[^}]+\})(,)',
        r'\1Arc::new(\2)\3',
        content
    )
    
    # Pattern 3: Multi-line closures need manual handling
    # We'll detect and wrap them
    lines = content.split('\n')
    result_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Check for .add( followed by a closure on the next line
        if re.match(r'^\s*(?:graph|branch_[abc]|branch|stats_branch|ml_branch)\.add\(\s*$', line):
            result_lines.append(line)
            i += 1
            
            # Check next line for closure start
            if i < len(lines) and re.match(r'^\s*\|.*\|\s*\{\s*$', lines[i]):
                # Add Arc::new( before the closure
                result_lines.append(re.sub(r'^(\s*)(\|.*)$', r'\1Arc::new(\2', lines[i]))
                i += 1
                
                # Find the end of the closure
                brace_count = 1
                while i < len(lines) and brace_count > 0:
                    current_line = lines[i]
                    brace_count += current_line.count('{') - current_line.count('}')
                    
                    if brace_count == 0:
                        # Add closing paren after the last }
                        result_lines.append(re.sub(r'^(\s*.*})(.*)$', r'\1)\2', current_line))
                    else:
                        result_lines.append(current_line)
                    i += 1
            else:
                # Not a closure, continue
                continue
        else:
            result_lines.append(line)
            i += 1
    
    content = '\n'.join(result_lines)
    
    # Pattern 4: .variants() with .collect() - need .map(Arc::new)
    content = re.sub(
        r'(\s*})\.collect\(\)',
        r'\1).map(Arc::new).collect()',
        content
    )
    
    # Pattern 5: Direct Vec with functions in variants
    content = re.sub(
        r'\.variants\(\s*vec!\[([^\]]+)\]',
        lambda m: '.variants(\n        vec![' + ', '.join(f'Arc::new({fn.strip()})' for fn in m.group(1).split(',')) + ']',
        content
    )
    
    return content

def add_arc_import(content):
    """Add Arc import if not present."""
    if 'use std::sync::Arc;' not in content:
        # Find the last use statement and add Arc import after it
        lines = content.split('\n')
        last_use_index = -1
        
        for i, line in enumerate(lines):
            if line.strip().startswith('use ') and not line.strip().startswith('use std::sync::Arc'):
                last_use_index = i
        
        if last_use_index >= 0:
            lines.insert(last_use_index + 1, 'use std::sync::Arc;')
        else:
            # Add at the top if no use statements found
            lines.insert(0, 'use std::sync::Arc;')
        
        content = '\n'.join(lines)
    
    return content

def fix_file(filepath):
    """Fix a single Rust file."""
    print(f"Fixing {filepath}")
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Add Arc import
    content = add_arc_import(content)
    
    # Fix function calls
    content = fix_function_calls(content)
    
    # Write back
    with open(filepath, 'w') as f:
        f.write(content)

def main():
    # Find all Rust files in examples and tests
    rust_files = []
    rust_files.extend(glob.glob('/workspaces/graph-sp/examples/rs/*.rs'))
    rust_files.extend(glob.glob('/workspaces/graph-sp/tests/*.rs'))
    
    # Skip files that are already working
    skip_files = ['simple_frontier_test.rs', 'radar_demo.rs']
    
    for filepath in rust_files:
        filename = os.path.basename(filepath)
        if filename not in skip_files:
            fix_file(filepath)
    
    print("Done! All files fixed.")

if __name__ == '__main__':
    main()