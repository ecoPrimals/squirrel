#!/usr/bin/env python3
"""
Analyze async_trait usage patterns in Squirrel codebase
"""
import re
from collections import defaultdict, Counter
from pathlib import Path

def analyze_async_trait_inventory(inventory_file):
    """Analyze async_trait inventory file"""
    
    # Storage
    by_crate = defaultdict(list)
    by_module = defaultdict(list)
    by_file = defaultdict(int)
    
    with open(inventory_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            
            # Parse: crates/path/to/file.rs:line_number:#[async_trait]
            parts = line.split(':')
            if len(parts) < 2:
                continue
            
            file_path = parts[0]
            
            # Extract crate and module
            path_parts = file_path.replace('crates/', '').split('/')
            if len(path_parts) < 2:
                continue
            
            crate = path_parts[0]
            module = '/'.join(path_parts[1:3]) if len(path_parts) > 2 else path_parts[1]
            
            by_crate[crate].append(file_path)
            by_module[f"{crate}/{module}"].append(file_path)
            by_file[file_path] += 1
    
    return by_crate, by_module, by_file

def print_analysis(by_crate, by_module, by_file):
    """Print analysis results"""
    
    print("=" * 80)
    print("ASYNC_TRAIT MIGRATION ANALYSIS")
    print("=" * 80)
    print()
    
    # Total count
    total = sum(len(files) for files in by_crate.values())
    print(f"Total async_trait instances: {total}")
    print()
    
    # By crate
    print("=" * 80)
    print("DISTRIBUTION BY CRATE (Top 15)")
    print("=" * 80)
    crate_counts = [(crate, len(files)) for crate, files in by_crate.items()]
    crate_counts.sort(key=lambda x: x[1], reverse=True)
    
    for crate, count in crate_counts[:15]:
        percentage = (count / total * 100) if total > 0 else 0
        print(f"{count:4d} instances ({percentage:5.1f}%)  {crate}")
    print()
    
    # By module
    print("=" * 80)
    print("HOT PATHS - MODULES WITH MOST ASYNC_TRAIT (Top 20)")
    print("=" * 80)
    module_counts = [(module, len(files)) for module, files in by_module.items()]
    module_counts.sort(key=lambda x: x[1], reverse=True)
    
    for module, count in module_counts[:20]:
        percentage = (count / total * 100) if total > 0 else 0
        print(f"{count:4d} instances ({percentage:5.1f}%)  {module}")
    print()
    
    # By file
    print("=" * 80)
    print("FILES WITH MOST ASYNC_TRAIT (Top 30)")
    print("=" * 80)
    file_counts = list(by_file.items())
    file_counts.sort(key=lambda x: x[1], reverse=True)
    
    for file_path, count in file_counts[:30]:
        print(f"{count:4d} instances  {file_path}")
    print()
    
    # Migration priorities
    print("=" * 80)
    print("RECOMMENDED MIGRATION PRIORITIES")
    print("=" * 80)
    print()
    print("Priority 1: Core MCP (Enhanced + Protocol)")
    mcp_count = by_crate.get('core', [])
    mcp_enhanced = [f for f in mcp_count if '/mcp/src/enhanced/' in f]
    mcp_protocol = [f for f in mcp_count if '/mcp/src/protocol/' in f]
    print(f"  Enhanced MCP:  ~{len(mcp_enhanced)} instances")
    print(f"  MCP Protocol:  ~{len(mcp_protocol)} instances")
    print()
    
    print("Priority 2: AI Tools")
    tools_count = by_crate.get('tools', [])
    ai_tools = [f for f in tools_count if '/ai-tools/' in f]
    print(f"  AI Tools:      ~{len(ai_tools)} instances")
    print()
    
    print("Priority 3: Integration & Plugins")
    integration_count = by_crate.get('integration', [])
    plugins_count = by_crate.get('core', [])
    plugins = [f for f in plugins_count if '/plugins/' in f]
    print(f"  Integration:   ~{len(integration_count)} instances")
    print(f"  Plugins:       ~{len(plugins)} instances")
    print()
    
    # Estimated effort
    print("=" * 80)
    print("ESTIMATED MIGRATION EFFORT")
    print("=" * 80)
    print(f"Total instances:     {total}")
    print(f"Average per day:     ~15-20 migrations")
    print(f"Estimated days:      {total // 15} - {total // 10} working days")
    print(f"Estimated weeks:     4-6 weeks (part-time)")
    print(f"Expected result:     <10 instances remaining (legitimate uses)")
    print(f"Expected reduction:  {total - 10} instances (97% reduction)")
    print()

if __name__ == "__main__":
    inventory_file = "async_trait_inventory.txt"
    
    try:
        by_crate, by_module, by_file = analyze_async_trait_inventory(inventory_file)
        print_analysis(by_crate, by_module, by_file)
        
        print("=" * 80)
        print("Analysis complete! Review priorities above.")
        print("=" * 80)
        
    except FileNotFoundError:
        print(f"Error: {inventory_file} not found")
        print("Run: grep -rn '#[async_trait]' crates --include='*.rs' > analysis/async_trait_inventory.txt")

