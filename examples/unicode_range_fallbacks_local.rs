use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Wrap};
use std::collections::HashMap;
use fontdb::Family;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Unicode Range Fallbacks with Local Fonts ===\n");
    
    // Create a new font system with local fonts
    let mut font_system = FontSystem::new();
    
    // Add local fonts from the fonts directory
    let fonts_dir = Path::new("fonts");
    println!("Loading fonts from {}:", fonts_dir.display());
    
    // Load the font files and get their IDs directly
    println!("  - Loading FiraMono-Medium");
    let firamono_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("FiraMono-Medium.ttf"))
    );
    
    println!("  - Loading Inter-Regular");
    let inter_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("Inter-Regular.ttf"))
    );
    
    println!("  - Loading NotoSans-Regular");
    let notosans_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("NotoSans-Regular.ttf"))
    );
    
    println!("  - Loading NotoSansArabic-Regular (or other non-Latin font)");
    let arabic_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("NotoSansArabic.ttf"))
    );
    
    // Store the font IDs for later use with range fallbacks
    let firamono_id = match firamono_ids.first() {
        Some(id) => *id,
        None => {
            println!("Failed to load FiraMono font");
            return Ok(());
        }
    };
    
    let inter_id = match inter_ids.first() {
        Some(id) => *id,
        None => {
            println!("Failed to load Inter font");
            return Ok(());
        }
    };
    
    let notosans_id = match notosans_ids.first() {
        Some(id) => *id,
        None => {
            println!("Failed to load NotoSans font");
            return Ok(());
        }
    };
    
    let arabic_id = match arabic_ids.first() {
        Some(id) => *id,
        None => {
            println!("Failed to load Arabic font");
            return Ok(());
        }
    };
    
    // Print the IDs that were returned
    println!("\nReturned Font IDs:");
    println!("  - FiraMono-Medium: {:?}", firamono_ids);
    println!("  - Inter-Regular: {:?}", inter_ids);  
    println!("  - NotoSans-Regular: {:?}", notosans_ids);
    println!("  - NotoSansArabic-Regular: {:?}", arabic_ids);
    
    // Get face information for our loaded fonts
    let mut firamono_family = String::new();
    let mut inter_family = String::new();
    let mut notosans_family = String::new(); 
    let mut arabic_family = String::new();
    
    let mut firamono_postscript = String::new();
    let mut inter_postscript = String::new();
    let mut notosans_postscript = String::new();
    let mut arabic_postscript = String::new();
    
    println!("\nFont details:");
    
    // Get FiraMono details
    if let Some(face) = font_system.db().face(firamono_id) {
        if let Some((name, _)) = face.families.first() {
            firamono_family = name.clone();
        }
        firamono_postscript = face.post_script_name.clone();
        println!("  - FiraMono: ID {}, family '{}', postscript '{}'", 
                 firamono_id, firamono_family, firamono_postscript);
    }
    
    // Get Inter details
    if let Some(face) = font_system.db().face(inter_id) {
        if let Some((name, _)) = face.families.first() {
            inter_family = name.clone();
        }
        inter_postscript = face.post_script_name.clone();
        println!("  - Inter: ID {}, family '{}', postscript '{}'", 
                 inter_id, inter_family, inter_postscript);
    }
    
    // Get NotoSans details
    if let Some(face) = font_system.db().face(notosans_id) {
        if let Some((name, _)) = face.families.first() {
            notosans_family = name.clone();
        }
        notosans_postscript = face.post_script_name.clone();
        println!("  - NotoSans: ID {}, family '{}', postscript '{}'", 
                 notosans_id, notosans_family, notosans_postscript);
    }
    
    // Get Arabic font details
    if let Some(face) = font_system.db().face(arabic_id) {
        if let Some((name, _)) = face.families.first() {
            arabic_family = name.clone();
        }
        arabic_postscript = face.post_script_name.clone();
        println!("  - Arabic font: ID {}, family '{}', postscript '{}'", 
                 arabic_id, arabic_family, arabic_postscript);
    }
    
    // Verify we found all our fonts
    if firamono_family.is_empty() || inter_family.is_empty() || notosans_family.is_empty() || arabic_family.is_empty() {
        println!("\nWARNING: One or more fonts are missing:");
        println!("  - FiraMono: {}", if firamono_family.is_empty() { "MISSING" } else { &firamono_family });
        println!("  - Inter: {}", if inter_family.is_empty() { "MISSING" } else { &inter_family });
        println!("  - NotoSans: {}", if notosans_family.is_empty() { "MISSING" } else { &notosans_family });
        println!("  - Arabic font: {}", if arabic_family.is_empty() { "MISSING" } else { &arabic_family });
    }
    
    // Configure Unicode range fallbacks using font IDs directly
    println!("\nConfiguring Unicode range fallbacks with font IDs:");
    println!("  - a-f: {} ({})", firamono_id, firamono_postscript);
    font_system.add_unicode_range_fallback('a', 'f', firamono_id);
    
    println!("  - g-r: {} ({})", inter_id, inter_postscript);
    font_system.add_unicode_range_fallback('g', 'r', inter_id);
    
    println!("  - s-z: {} ({})", notosans_id, notosans_postscript);
    font_system.add_unicode_range_fallback('s', 'z', notosans_id);
    
    // Create a buffer with Latin text that will use all three ranges
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 24.0));
    
    // Create a test string that uses all ranges (a-f, g-r, s-z)
    let text = "abcdefghijklmnopqrstuvwxyz";
    
    // Create attributes with the Arabic font as the primary font
    // Since Arabic font doesn't have Latin glyphs, fallbacks will be used
    println!("\nUsing '{}' as primary font", arabic_family);
    let attrs = Attrs::new().family(Family::Name(&arabic_family));
    
    // Set the text with our attributes
    buffer.set_text(&mut font_system, text, &attrs, Shaping::Advanced);
    
    // Set wrap to None to allow all text on one line
    buffer.set_wrap(&mut font_system, Wrap::None);
    
    // Print the buffer for visual inspection
    println!("\nBuffer with text: {}", text);
    
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
                format!("{} ({})", name, face.post_script_name)
            } else {
                face.post_script_name.clone()
            }
        } else {
            "unknown".to_string()
        };
        
        char_to_font.insert(character.to_string(), font_name);
    }
    
    // Print which font was used for each character, grouped by expected ranges
    println!("\nFont usage by character:");
    
    // Create a function to determine the expected font for each character
    let get_expected_font = |c: char| -> String {
        if ('a'..='f').contains(&c) {
            format!("{} ({})", firamono_family, firamono_postscript)
        } else if ('g'..='r').contains(&c) {
            format!("{} ({})", inter_family, inter_postscript)
        } else if ('s'..='z').contains(&c) {
            format!("{} ({})", notosans_family, notosans_postscript)
        } else {
            "(none expected)".to_string()
        }
    };
    
    // Display used font and expected font for each character
    for c in text.chars() {
        let c_str = c.to_string();
        let used_font = match char_to_font.get(&c_str) {
            Some(font_name) => font_name,
            None => "not found"
        };
        let expected_font = get_expected_font(c);
        println!("'{}' -> Used: {}, Expected: {}", c, used_font, expected_font);
    }
    
    // Add a more complex text with mixed fallbacks
    println!("\n=== Testing with a sentence ===\n");
    
    let sentence = "The quick brown fox jumps over the lazy dog.";
    buffer.set_text(&mut font_system, sentence, &attrs, Shaping::Advanced);
    
    // Get font usage for the sentence
    let layout = buffer.layout_runs().next().unwrap();
    let mut char_to_font = HashMap::new();
    
    for glyph in layout.glyphs {
        let char_range = glyph.start..glyph.end;
        let character = &sentence[char_range.clone()];
        let font_id = glyph.font_id;
        
        // Get the font name if available
        let font_name = if let Some(face) = font_system.db().face(font_id) {
            if let Some((name, _)) = face.families.first() {
                format!("{} ({})", name, face.post_script_name)
            } else {
                face.post_script_name.clone()
            }
        } else {
            "unknown".to_string()
        };
        
        char_to_font.insert(character.to_string(), font_name);
    }
    
    println!("Sentence: {}", sentence);
    println!("\nFont usage by character:");
    for c in sentence.chars() {
        let c_str = c.to_string();
        let used_font = match char_to_font.get(&c_str) {
            Some(font_name) => font_name,
            None => "not found"
        };
        let expected_font = get_expected_font(c);
        println!("'{}' -> Used: {}, Expected: {}", c, used_font, expected_font);
    }
    
    // Add instructions for debug logging
    println!("\n=== Debug Logging Instructions ===");
    println!("To see detailed debug logs of the fallback process:");
    println!("Run the example with: RUST_LOG=debug cargo run --example unicode_range_fallbacks_local");
    println!("This will show how the fallback mechanism works internally.");
    
    // Create a small string with one character from each range
    let small_text = "afntz";
    println!("\nTest string: {} (characters from different ranges)", small_text);
    buffer.set_text(&mut font_system, small_text, &attrs, Shaping::Advanced);
    
    // Get font usage for the small test string
    let layout = buffer.layout_runs().next().unwrap();
    let mut char_to_font = HashMap::new();
    
    for glyph in layout.glyphs {
        let char_range = glyph.start..glyph.end;
        let character = &small_text[char_range.clone()];
        let font_id = glyph.font_id;
        
        // Get the font name if available
        let font_name = if let Some(face) = font_system.db().face(font_id) {
            if let Some((name, _)) = face.families.first() {
                format!("{} ({})", name, face.post_script_name)
            } else {
                face.post_script_name.clone()
            }
        } else {
            "unknown".to_string()
        };
        
        char_to_font.insert(character.to_string(), font_name);
    }
    
    println!("\nFont usage for test string:");
    for c in small_text.chars() {
        let c_str = c.to_string();
        let used_font = match char_to_font.get(&c_str) {
            Some(font_name) => font_name,
            None => "not found"
        };
        let expected_font = get_expected_font(c);
        println!("'{}' -> Used: {}, Expected: {}", c, used_font, expected_font);
    }
    
    Ok(())
} 