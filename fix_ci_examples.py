#!/usr/bin/env python3

import os
import re

# List of examples that CI tries to run
CI_EXAMPLES = [
    'comprehensive_demo.rs',
    'output_access_demo.rs', 
    'parallel_execution_demo.rs',
    'per_node_output_access.rs',
    'tuple_api_demo.rs',
    'variant_demo_full.rs'
]

def add_imports(content):
    """Add necessary imports."""
    if 'use std::sync::Arc;' not in content:
        # Find existing use statements
        lines = content.split('\n')
        insert_pos = 0
        
        for i, line in enumerate(lines):
            if line.strip().startswith('use '):
                insert_pos = i + 1
        
        lines.insert(insert_pos, 'use std::sync::Arc;')
        content = '\n'.join(lines)
    
    # Also add NodeFunction if using variants
    if 'variants(' in content and 'NodeFunction' not in content:
        content = content.replace('use dagex::{', 'use dagex::{NodeFunction, ')
    
    return content

def fix_simple_add_calls(content):
    """Fix .add() calls with function names."""
    # Pattern: .add(function_name, 
    content = re.sub(
        r'(\.(add)\(\s*)([a-z_][a-z0-9_]*)(,)',
        r'\1Arc::new(\3)\4',
        content
    )
    return content

def fix_closure_wrapping(content):
    """Fix closures that need Arc::new wrapping."""
    lines = content.split('\n')
    result = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Look for .add( on its own line
        if re.search(r'\.(add)\(\s*$', line):
            result.append(line)
            i += 1
            
            # Check if next line starts a closure
            if i < len(lines) and re.match(r'^\s*\|', lines[i]):
                # Wrap with Arc::new
                closure_line = lines[i]
                indent_match = re.match(r'^(\s*)', closure_line)
                indent = indent_match.group(1) if indent_match else ''
                
                wrapped_line = indent + 'Arc::new(' + closure_line.strip()
                result.append(wrapped_line)
                i += 1
                
                # Find the closing brace and add closing paren
                brace_count = 1 if '{' in wrapped_line else 0
                
                while i < len(lines) and brace_count > 0:
                    current_line = lines[i]
                    brace_count += current_line.count('{') - current_line.count('}')
                    
                    if brace_count == 0:
                        # Add closing paren
                        if current_line.strip().endswith('},'):
                            result.append(current_line.replace('},', '}),'))
                        elif current_line.strip().endswith('}'):
                            result.append(current_line + ')')
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

def convert_variants_patterns(content):
    """Convert .map().collect() patterns to explicit Vec<NodeFunction>."""
    
    # Look for the pattern: .map(|&factor| { ... }).collect()
    # and replace with explicit Vec<NodeFunction>
    
    # This is complex to do with regex, so let's do a simple replacement for now
    # that handles the common cases
    if '}).collect()' in content and 'variants(' in content:
        # Just add .map(|f| Arc::new(f) as NodeFunction) before .collect()
        content = content.replace('}).collect()', '}).map(|f| Arc::new(f) as dagex::NodeFunction).collect()')
    
    return content

def fix_file(filepath):
    """Fix a single Rust example file."""
    print(f"Fixing {filepath}")
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    # Apply all fixes
    content = add_imports(content)
    content = fix_simple_add_calls(content)
    content = fix_closure_wrapping(content)
    content = convert_variants_patterns(content)
    
    with open(filepath, 'w') as f:
        f.write(content)

def main():
    """Fix all CI examples."""
    base_path = '/workspaces/graph-sp/examples/rs'
    
    for example_name in CI_EXAMPLES:
        filepath = os.path.join(base_path, example_name)
        if os.path.exists(filepath):
            fix_file(filepath)
        else:
            print(f"Warning: {filepath} not found")
    
    print("Done fixing CI examples!")

if __name__ == '__main__':
    main()