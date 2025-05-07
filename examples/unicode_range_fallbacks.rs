use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Wrap};
use std::collections::HashMap;
use fontdb::Family;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First test with system fonts
    test_with_system_fonts()?;
    
    // Second test with local fonts
    test_with_local_fonts()?;
    
    Ok(())
}

fn test_with_system_fonts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Test 1: Using System Fonts ===\n");
    
    // Create a new font system
    let mut font_system = FontSystem::new();
    
    // Configure Unicode range fallbacks with fonts available on macOS
    // Using fonts from the provided system font list
    font_system.add_unicode_range_fallback('\u{0370}', '\u{03FF}', "Times New Roman"); // Greek
    font_system.add_unicode_range_fallback('\u{0600}', '\u{06FF}', "Arial"); // Arabic
    font_system.add_unicode_char_fallback('α', "Arial"); // Specific character
    font_system.add_unicode_block_fallback("Emoji", "Apple Color Emoji")?;
    
    // Create a buffer with some text containing characters from different scripts
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    
    // Add some text with mixed scripts
    let text = "Hello, world! こんにちは 你好 α β γ 😀 😎 👍";
    
    // Create attributes with a specific font family
    let attrs = Attrs::new().family(Family::Name("Times New Roman"));
    
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced);
    
    // Set wrap to None to allow all text on one line
    buffer.set_wrap(&mut font_system, Wrap::None);
    
    // Print the buffer for visual inspection
    println!("Buffer with text: {}", text);
    
    // Get information about which fonts were used for which characters
    let layout = buffer.layout_runs().next().unwrap();
    let mut char_to_font = HashMap::new();
    
    for glyph in layout.glyphs {
        let char_range = glyph.start..glyph.end;
        let character = &text[char_range.clone()];
        let font_id = glyph.font_id;
        
        // Get the font name if available
        let font_name = if let Some(face) = font_system.db().face(font_id) {
            if let Some((name, _)) = face.families.first() {
                name.clone()
            } else {
                face.post_script_name.clone()
            }
        } else {
            "unknown".to_string()
        };
        
        char_to_font.insert(character.to_string(), font_name);
    }
    
    // Print which font was used for each character
    println!("\nFont usage by character:");
    for (character, font) in char_to_font {
        println!("'{}' -> {}", character, font);
    }
    
    Ok(())
}

fn test_with_local_fonts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Test 2: Using Local Fonts ===\n");
    
    // Create a new font system with local fonts
    let mut font_system = FontSystem::new();
    
    // Add local fonts from the fonts directory
    let fonts_dir = Path::new("fonts");
    
    // Load the font files
    let firamono = fontdb::Source::File(fonts_dir.join("firamono-medium.ttf"));
    let inter = fontdb::Source::File(fonts_dir.join("inter-regular.ttf"));
    let notosans = fontdb::Source::File(fonts_dir.join("notosans-regular.ttf"));
    
    font_system.db_mut().load_font_source(firamono);
    font_system.db_mut().load_font_source(inter);
    font_system.db_mut().load_font_source(notosans);
    
    // Configure Unicode range fallbacks for Latin characters
    // a-f for Fira Mono Medium
    font_system.add_unicode_range_fallback('a', 'f', "Fira Mono Medium");
    // g-r for Inter Regular
    font_system.add_unicode_range_fallback('g', 'r', "Inter Regular");
    // s-z for Noto Sans Regular
    font_system.add_unicode_range_fallback('s', 'z', "Noto Sans Regular");
    
    // Create a buffer with some text containing Latin characters
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    
    // Add text that uses all three ranges
    let text = "abcdefghijklmnopqrstuvwxyz";
    
    // Use default attributes
    let attrs = Attrs::new();
    
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced);
    
    // Set wrap to None to allow all text on one line
    buffer.set_wrap(&mut font_system, Wrap::None);
    
    // Print the buffer for visual inspection
    println!("Buffer with text: {}", text);
    
    // Get information about which fonts were used for which characters
    let layout = buffer.layout_runs().next().unwrap();
    let mut char_to_font = HashMap::new();
    
    for glyph in layout.glyphs {
        let char_range = glyph.start..glyph.end;
        let character = &text[char_range.clone()];
        let font_id = glyph.font_id;
        
        // Get the font name if available
        let font_name = if let Some(face) = font_system.db().face(font_id) {
            if let Some((name, _)) = face.families.first() {
                name.clone()
            } else {
                face.post_script_name.clone()
            }
        } else {
            "unknown".to_string()
        };
        
        char_to_font.insert(character.to_string(), font_name);
    }
    
    // Print which font was used for each character
    println!("\nFont usage by character:");
    for (character, font) in char_to_font {
        println!("'{}' -> {}", character, font);
    }
    
    Ok(())
} 