use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Wrap};
use std::collections::HashMap;
use fontdb::{Family, Style, Weight};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_with_style_and_weight()?;
    Ok(())
}

fn test_with_style_and_weight() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Test: Using Unicode Range Fallbacks with Style and Weight ===\n");
    
    // Create a new font system
    let mut font_system = FontSystem::new();
    
    // Add local fonts from the fonts directory
    let fonts_dir = Path::new("fonts");
    
    // Load font files with different weights and styles
    // Note: These paths are examples - replace with actual font paths in your setup
    let regular = fontdb::Source::File(fonts_dir.join("inter-regular.ttf"));
    let bold = fontdb::Source::File(fonts_dir.join("inter-bold.ttf"));
    let italic = fontdb::Source::File(fonts_dir.join("inter-italic.ttf"));
    let bold_italic = fontdb::Source::File(fonts_dir.join("inter-bolditalic.ttf"));
    
    font_system.db_mut().load_font_source(regular);
    font_system.db_mut().load_font_source(bold);
    font_system.db_mut().load_font_source(italic);
    font_system.db_mut().load_font_source(bold_italic);
    
    // Configure Unicode range fallbacks with specific weights and styles
    
    // For Latin uppercase (A-Z), use regular weight
    font_system.add_unicode_range_fallback_name_with_style(
        'A', 'Z', 
        "Inter", 
        Some(Weight::NORMAL), 
        Some(Style::Normal)
    )?;
    
    // For Latin lowercase (a-z), use bold weight
    font_system.add_unicode_range_fallback_name_with_style(
        'a', 'z', 
        "Inter", 
        Some(Weight::BOLD), 
        Some(Style::Normal)
    )?;
    
    // For digits (0-9), use italic style
    font_system.add_unicode_range_fallback_name_with_style(
        '0', '9', 
        "Inter", 
        Some(Weight::NORMAL), 
        Some(Style::Italic)
    )?;
    
    // For punctuation, use bold italic style
    font_system.add_unicode_range_fallback_name_with_style(
        '!', '/',
        "Inter",
        Some(Weight::BOLD),
        Some(Style::Italic)
    )?;
    
    // Example of adding a single character fallback
    font_system.add_unicode_char_fallback_name('@', "Inter")?;
    
    // Create multiple buffers to test different text with different requested styles
    
    // Test 1: Regular text with regular style request (should match the regular font)
    let mut buffer1 = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    let text1 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let attrs1 = Attrs::new()
        .style(Style::Normal)
        .weight(Weight::NORMAL);
    
    buffer1.set_text(&mut font_system, text1, &attrs1, Shaping::Advanced);
    buffer1.set_wrap(&mut font_system, Wrap::None);
    
    // Test 2: Lowercase text with bold style request (should match the bold font)
    let mut buffer2 = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    let text2 = "abcdefghijklmnopqrstuvwxyz";
    let attrs2 = Attrs::new()
        .style(Style::Normal)
        .weight(Weight::BOLD);
    
    buffer2.set_text(&mut font_system, text2, &attrs2, Shaping::Advanced);
    buffer2.set_wrap(&mut font_system, Wrap::None);
    
    // Test 3: Digits with italic style request (should match the italic font)
    let mut buffer3 = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    let text3 = "0123456789";
    let attrs3 = Attrs::new()
        .style(Style::Italic)
        .weight(Weight::NORMAL);
    
    buffer3.set_text(&mut font_system, text3, &attrs3, Shaping::Advanced);
    buffer3.set_wrap(&mut font_system, Wrap::None);
    
    // Test 4: Punctuation with bold italic style
    let mut buffer4 = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    let text4 = "!\"#$%&'()*+,-./";
    let attrs4 = Attrs::new()
        .style(Style::Italic)
        .weight(Weight::BOLD);
    
    buffer4.set_text(&mut font_system, text4, &attrs4, Shaping::Advanced);
    buffer4.set_wrap(&mut font_system, Wrap::None);
    
    // Print the buffers for visual inspection
    println!("Buffer 1 (Regular uppercase with regular style):");
    print_font_usage(&mut font_system, &buffer1, text1);
    
    println!("\nBuffer 2 (Lowercase with bold style):");
    print_font_usage(&mut font_system, &buffer2, text2);
    
    println!("\nBuffer 3 (Digits with italic style):");
    print_font_usage(&mut font_system, &buffer3, text3);
    
    println!("\nBuffer 4 (Punctuation with bold italic style):");
    print_font_usage(&mut font_system, &buffer4, text4);
    
    Ok(())
}

// Helper function to print which font was used for each character
fn print_font_usage(font_system: &mut FontSystem, buffer: &Buffer, text: &str) {
    println!("Text: {}", text);
    
    if let Some(layout) = buffer.layout_runs().next() {
        let mut char_to_font = HashMap::new();
        
        for glyph in layout.glyphs.iter() {
            let char_range = glyph.start..glyph.end;
            if char_range.start >= text.len() || char_range.end > text.len() {
                continue;
            }
            
            let character = &text[char_range.clone()];
            let font_id = glyph.font_id;
            
            // Get the font name and style information if available
            let font_info = if let Some(face) = font_system.db().face(font_id) {
                let name = if let Some((name, _)) = face.families.first() {
                    name.clone()
                } else {
                    face.post_script_name.clone()
                };
                
                // Include the weight and style in the output
                let weight = match face.weight {
                    Weight::NORMAL => "Regular",
                    Weight::BOLD => "Bold",
                    _ => "Other",
                };
                
                let style = match face.style {
                    Style::Normal => "Normal",
                    Style::Italic => "Italic",
                    Style::Oblique => "Oblique",
                };
                
                format!("{} ({} {})", name, weight, style)
            } else {
                "unknown".to_string()
            };
            
            char_to_font.insert(character.to_string(), font_info);
        }
        
        // Print which font was used for each character
        println!("Font usage by character:");
        for (character, font) in char_to_font {
            println!("'{}' -> {}", character, font);
        }
    } else {
        println!("No layout runs found!");
    }
} 