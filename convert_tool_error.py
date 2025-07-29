#!/usr/bin/env python3
"""
Script to convert ToolError usages to ErrorData usages.

This script converts:
- ToolError::ExecutionError(msg) -> ErrorData::new(ErrorCode::INTERNAL_ERROR, msg, None)
- ToolError::InvalidParameters(msg) -> ErrorData::new(ErrorCode::INVALID_PARAMS, msg, None)
- ToolError::NotFound(msg) -> ErrorData::new(ErrorCode::INVALID_REQUEST, msg, None)
- ToolError::SchemaError(msg) -> ErrorData::new(ErrorCode::INVALID_PARAMS, msg, None)
- Result<..., ToolError> -> Result<..., ErrorData>
"""

import os
import re
import subprocess
import sys
from pathlib import Path

def find_rust_files():
    """Find all Rust files in the project."""
    result = subprocess.run(['find', './crates', '-name', '*.rs', '-type', 'f'], 
                          capture_output=True, text=True)
    if result.returncode != 0:
        print("Error finding Rust files")
        sys.exit(1)
    
    return [f.strip() for f in result.stdout.split('\n') if f.strip()]

def convert_tool_errors(content):
    """Convert ToolError usages to ErrorData usages."""
    changes_made = False
    original_content = content
    
    # First, handle Result type signatures
    patterns = [
        (r'Result<([^,<>]+(?:<[^<>]*>)?[^,<>]*),\s*ToolError>', r'Result<\1, ErrorData>'),
        (r'-> Result<Vec<Content>, ToolError>', r'-> Result<Vec<Content>, ErrorData>'),
        (r'-> Result<([^,<>]+(?:<[^<>]*>)?[^,<>]*), ToolError>', r'-> Result<\1, ErrorData>'),
    ]
    
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            content = new_content
            changes_made = True
    
    # Handle ToolError constructors with simple arguments
    patterns = [
        (r'ToolError::ExecutionError\(([^()]+)\)', r'ErrorData::new(ErrorCode::INTERNAL_ERROR, \1, None)'),
        (r'ToolError::InvalidParameters\(([^()]+)\)', r'ErrorData::new(ErrorCode::INVALID_PARAMS, \1, None)'),
        (r'ToolError::NotFound\(([^()]+)\)', r'ErrorData::new(ErrorCode::INVALID_REQUEST, \1, None)'),
        (r'ToolError::SchemaError\(([^()]+)\)', r'ErrorData::new(ErrorCode::INVALID_PARAMS, \1, None)'),
    ]
    
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            content = new_content
            changes_made = True
    
    # Handle ToolError constructors with format! calls
    patterns = [
        (r'ToolError::ExecutionError\(format!\(([^)]+)\)\)', r'ErrorData::new(ErrorCode::INTERNAL_ERROR, format!(\1), None)'),
        (r'ToolError::InvalidParameters\(format!\(([^)]+)\)\)', r'ErrorData::new(ErrorCode::INVALID_PARAMS, format!(\1), None)'),
        (r'ToolError::NotFound\(format!\(([^)]+)\)\)', r'ErrorData::new(ErrorCode::INVALID_REQUEST, format!(\1), None)'),
        (r'ToolError::SchemaError\(format!\(([^)]+)\)\)', r'ErrorData::new(ErrorCode::INVALID_PARAMS, format!(\1), None)'),
    ]
    
    for pattern, replacement in patterns:
        new_content = re.sub(pattern, replacement, content)
        if new_content != content:
            content = new_content
            changes_made = True
    
    # Handle use statements with multiple imports
    pattern_use2 = r'use mcp_core::\{([^}]*?)ToolError([^}]*?)\};'
    def replace_use_multi(match):
        before = match.group(1)
        after = match.group(2)
        # Remove ToolError from the import list
        imports = (before + after).split(',')
        imports = [imp.strip() for imp in imports if imp.strip() and imp.strip() != 'ToolError']
        if imports:
            return f'use mcp_core::{{{", ".join(imports)}}};'
        else:
            return ''
    
    new_content = re.sub(pattern_use2, replace_use_multi, content)
    if new_content != content:
        content = new_content
        changes_made = True
    
    # Handle use statements with handler:: prefix
    pattern_use3 = r'use mcp_core::handler::\{([^}]*?)ToolError([^}]*?)\};'
    def replace_use_handler(match):
        before = match.group(1)
        after = match.group(2)
        # Remove ToolError from the import list
        imports = (before + after).split(',')
        imports = [imp.strip() for imp in imports if imp.strip() and imp.strip() != 'ToolError']
        if imports:
            return f'use mcp_core::handler::{{{", ".join(imports)}}};'
        else:
            return ''
    
    new_content = re.sub(pattern_use3, replace_use_handler, content)
    if new_content != content:
        content = new_content
        changes_made = True
    
    return content, changes_made

def add_imports_if_needed(content):
    """Add ErrorData and ErrorCode imports if they are used but not imported."""
    if 'ErrorData' in content or 'ErrorCode' in content:
        # Check if already imported
        if 'use rmcp::model::{ErrorData, ErrorCode}' in content or 'use rmcp::model::ErrorData' in content:
            return content
        
        lines = content.split('\n')
        
        # Find the best place to add the import
        import_line = None
        for i, line in enumerate(lines):
            if line.startswith('use '):
                import_line = i + 1
        
        if import_line is not None:
            lines.insert(import_line, 'use rmcp::model::{ErrorData, ErrorCode};')
        else:
            # Add at the top
            lines.insert(0, 'use rmcp::model::{ErrorData, ErrorCode};')
        
        return '\n'.join(lines)
    
    return content

def process_file(file_path):
    """Process a single Rust file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Skip files that don't contain ToolError
        if 'ToolError' not in content:
            return False
        
        # Skip files in target directory
        if '/target/' in file_path:
            return False
        
        original_content = content
        
        # Convert ToolError usages
        content, changes_made = convert_tool_errors(content)
        
        if changes_made:
            # Add imports if needed
            content = add_imports_if_needed(content)
            
            # Write back to file
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            
            print(f"Updated: {file_path}")
            return True
        
        return False
        
    except Exception as e:
        print(f"Error processing {file_path}: {e}")
        return False

def main():
    """Main function."""
    print("Converting ToolError usages to ErrorData...")
    
    # Find all Rust files
    rust_files = find_rust_files()
    print(f"Found {len(rust_files)} Rust files")
    
    updated_count = 0
    for file_path in rust_files:
        if process_file(file_path):
            updated_count += 1
    
    print(f"Updated {updated_count} files")
    
    # Run cargo fmt to format the code
    print("Running cargo fmt...")
    result = subprocess.run(['cargo', 'fmt'], capture_output=True, text=True)
    if result.returncode != 0:
        print(f"cargo fmt failed: {result.stderr}")
    else:
        print("cargo fmt completed successfully")

if __name__ == '__main__':
    main()
