#!/usr/bin/env python3

import re
import glob

def fix_add_closures(content):
    """Fix .add() calls with closures that need Arc::new wrapping."""
    
    # Split into lines for easier processing
    lines = content.split('\n')
    result = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Look for .add( followed by a closure
        if re.search(r'\.(add|variants)\(\s*$', line):
            result.append(line)
            i += 1
            
            # Check if next line starts a closure
            if i < len(lines) and re.match(r'^\s*\|', lines[i]):
                # This is a closure that needs Arc::new wrapping
                closure_line = lines[i]
                # Add Arc::new( at the start
                indented = re.match(r'^(\s*)', closure_line).group(1)
                wrapped_start = closure_line.replace(indented, indented + 'Arc::new(', 1)
                result.append(wrapped_start)
                i += 1
                
                # Find the end of the closure and add closing paren
                brace_count = 1 if '{' in wrapped_start else 0
                
                while i < len(lines):
                    current_line = lines[i]
                    brace_count += current_line.count('{') - current_line.count('}')
                    
                    if brace_count == 0:
                        # This should be the line with the closing }
                        if current_line.strip().endswith('},'):
                            result.append(current_line.replace('},', '}),'))
                        elif current_line.strip().endswith('}'):
                            result.append(current_line + ')')
                        else:
                            # Handle other patterns
                            result.append(re.sub(r'^(\s*\s*})(.*)$', r'\1)\2', current_line))
                        i += 1
                        break
                    else:
                        result.append(current_line)
                        i += 1
            else:
                # Not a closure, continue normally
                continue
        else:
            result.append(line)
            i += 1
    
    return '\n'.join(result)

def fix_function_names(content):
    """Fix function name arguments that need Arc::new wrapping."""
    
    # Pattern for function names passed to .add()
    patterns = [
        (r'(\.(add|variants)\(\s*)([a-z_][a-z0-9_]*)(,)', r'\1Arc::new(\3)\4'),
    ]
    
    for pattern, replacement in patterns:
        content = re.sub(pattern, replacement, content)
    
    return content

def ensure_arc_import(content):
    """Ensure Arc is imported."""
    if 'use std::sync::Arc;' not in content:
        # Find a good place to add the import
        lines = content.split('\n')
        
        # Look for existing use statements
        insert_pos = 0
        for i, line in enumerate(lines):
            if line.strip().startswith('use '):
                insert_pos = i + 1
        
        # Insert the Arc import
        lines.insert(insert_pos, 'use std::sync::Arc;')
        content = '\n'.join(lines)
    
    return content

def fix_file(filepath):
    """Fix all Arc::new issues in a file."""
    print(f"Fixing {filepath}")
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Apply fixes
    content = ensure_arc_import(content)
    content = fix_function_names(content)
    content = fix_add_closures(content)
    
    with open(filepath, 'w') as f:
        f.write(content)

def main():
    # Process all rust files that need fixing
    files = [
        '/workspaces/graph-sp/examples/rs/tuple_api_demo.rs',
        '/workspaces/graph-sp/examples/rs/parallel_execution_demo.rs', 
        '/workspaces/graph-sp/examples/rs/parallel_timing_demo.rs',
        '/workspaces/graph-sp/examples/rs/output_access_demo.rs',
        '/workspaces/graph-sp/examples/rs/per_node_output_access.rs',
        '/workspaces/graph-sp/examples/rs/comprehensive_demo.rs',
        '/workspaces/graph-sp/tests/integration_tests.rs'
    ]
    
    for filepath in files:
        fix_file(filepath)
    
    print("All files fixed!")

if __name__ == '__main__':
    main()