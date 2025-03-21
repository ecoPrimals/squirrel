# Header Widget

## Description
The header widget provides a visually distinct section header using box drawing characters. It creates a bordered box around the header text to make it stand out in the terminal interface.

## Visual Example
```
╭────────────────╮
│ Header Title   │
╰────────────────╯
```

## Implementation Details

### Function Signature
```rust
pub fn print_header(&mut self, text: &str) -> Result<()>
```

### Features
- Box drawing characters for visual appeal
- Bold text formatting
- Blue color highlighting
- Automatic width adjustment based on text length
- Proper indentation support
- Error handling for IO operations

### Technical Requirements
- Uses crossterm for terminal manipulation
- Supports Unicode box drawing characters
- Handles multi-byte characters correctly
- Maintains consistent spacing

## Usage Guidelines
1. Use for major section divisions
2. Keep header text concise
3. Ensure proper indentation level
4. Handle potential errors

## Example Usage
```rust
ui.print_header("Configuration Settings");
```

## Future Improvements
1. **Customization**
   - Support custom colors
   - Allow different box styles
   - Support multi-line headers
   - Add optional icons/symbols

2. **Layout**
   - Support dynamic width
   - Add alignment options
   - Support header groups
   - Add collapsible headers

3. **Styling**
   - Add gradient color support
   - Support custom border styles
   - Add shadow effects
   - Support background colors

4. **Accessibility**
   - Add screen reader hints
   - Support high contrast mode
   - Add alternative visual styles

## Error Handling
- Handles IO errors gracefully
- Reports formatting errors
- Maintains terminal state on failure
- Provides clear error messages

## Testing
- Visual appearance tests
- Unicode handling tests
- Error condition tests
- Indentation tests 