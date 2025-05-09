use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, Wrap};
use std::collections::HashMap;
use fontdb::{Family, Style, Weight};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Unicode Range Fallbacks with Style and Weight ===\n");
    
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
    
    println!("  - Loading NotoSansArabic-Regular");
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
    
    let mut firamono_weight = Weight::NORMAL;
    let mut inter_weight = Weight::NORMAL;
    let mut notosans_weight = Weight::NORMAL;
    let mut arabic_weight = Weight::NORMAL;
    
    let mut firamono_style = Style::Normal;
    let mut inter_style = Style::Normal;
    let mut notosans_style = Style::Normal;
    let mut arabic_style = Style::Normal;
    
    println!("\nFont details:");
    
    // Get FiraMono details
    if let Some(face) = font_system.db().face(firamono_id) {
        if let Some((name, _)) = face.families.first() {
            firamono_family = name.clone();
        }
        firamono_postscript = face.post_script_name.clone();
        firamono_weight = face.weight;
        firamono_style = face.style;
        println!("  - FiraMono: ID {}, family '{}', postscript '{}', weight {:?}, style {:?}", 
                 firamono_id, firamono_family, firamono_postscript, firamono_weight, firamono_style);
    }
    
    // Get Inter details
    if let Some(face) = font_system.db().face(inter_id) {
        if let Some((name, _)) = face.families.first() {
            inter_family = name.clone();
        }
        inter_postscript = face.post_script_name.clone();
        inter_weight = face.weight;
        inter_style = face.style;
        println!("  - Inter: ID {}, family '{}', postscript '{}', weight {:?}, style {:?}", 
                 inter_id, inter_family, inter_postscript, inter_weight, inter_style);
    }
    
    // Get NotoSans details
    if let Some(face) = font_system.db().face(notosans_id) {
        if let Some((name, _)) = face.families.first() {
            notosans_family = name.clone();
        }
        notosans_postscript = face.post_script_name.clone();
        notosans_weight = face.weight;
        notosans_style = face.style;
        println!("  - NotoSans: ID {}, family '{}', postscript '{}', weight {:?}, style {:?}", 
                 notosans_id, notosans_family, notosans_postscript, notosans_weight, notosans_style);
    }
    
    // Get Arabic font details
    if let Some(face) = font_system.db().face(arabic_id) {
        if let Some((name, _)) = face.families.first() {
            arabic_family = name.clone();
        }
        arabic_postscript = face.post_script_name.clone();
        arabic_weight = face.weight;
        arabic_style = face.style;
        println!("  - Arabic font: ID {}, family '{}', postscript '{}', weight {:?}, style {:?}", 
                 arabic_id, arabic_family, arabic_postscript, arabic_weight, arabic_style);
    }
    
    // Verify we found all our fonts
    if firamono_family.is_empty() || inter_family.is_empty() || notosans_family.is_empty() || arabic_family.is_empty() {
        println!("\nWARNING: One or more fonts are missing:");
        println!("  - FiraMono: {}", if firamono_family.is_empty() { "MISSING" } else { &firamono_family });
        println!("  - Inter: {}", if inter_family.is_empty() { "MISSING" } else { &inter_family });
        println!("  - NotoSans: {}", if notosans_family.is_empty() { "MISSING" } else { &notosans_family });
        println!("  - Arabic font: {}", if arabic_family.is_empty() { "MISSING" } else { &arabic_family });
    }
    
    // Configure Unicode range fallbacks using font IDs and styles/weights
    println!("\nConfiguring Unicode range fallbacks with font IDs and styles/weights:");
    
    // a-f: FiraMono with Medium weight, Normal style
    println!("  - a-f: {} ({}) with weight {:?}, style {:?}", 
              firamono_id, firamono_postscript, firamono_weight, firamono_style);
    font_system.add_unicode_range_fallback_with_style(
        'a', 'f', firamono_id, Some(firamono_weight), Some(firamono_style)
    );
    
    // g-r: Inter with Regular weight, Normal style
    println!("  - g-r: {} ({}) with weight {:?}, style {:?}", 
              inter_id, inter_postscript, inter_weight, inter_style);
    font_system.add_unicode_range_fallback_with_style(
        'g', 'r', inter_id, Some(inter_weight), Some(inter_style)
    );
    
    // s-z: NotoSans with Regular weight, Normal style
    println!("  - s-z: {} ({}) with weight {:?}, style {:?}", 
              notosans_id, notosans_postscript, notosans_weight, notosans_style);
    font_system.add_unicode_range_fallback_with_style(
        's', 'z', notosans_id, Some(notosans_weight), Some(notosans_style)
    );
    
    // Test with normal style and weight
    /*println!("\n=== Test 1: Default text with matching weights and styles ===");
    test_with_style_weight(
        &mut font_system, 
        "abcdefghijklmnopqrstuvwxyz", 
        &arabic_family, 
        Style::Normal, 
        Weight::NORMAL,
        &firamono_family, &inter_family, &notosans_family, 
        &firamono_postscript, &inter_postscript, &notosans_postscript
    )?;
    
    // Test with bold weight (should still match but with larger weight difference)
    println!("\n=== Test 2: Bold text with no exact weight match ===");
    test_with_style_weight(
        &mut font_system, 
        "abcdefghijklmnopqrstuvwxyz", 
        &arabic_family, 
        Style::Normal, 
        Weight::BOLD,
        &firamono_family, &inter_family, &notosans_family, 
        &firamono_postscript, &inter_postscript, &notosans_postscript
    )?;*/
    
    // Now clear the fallbacks and add new ones with specific style/weight for first test
    println!("\nClearing and reconfiguring fallbacks for specific weight/style matching:");
    
    // Configure a new set of fallbacks that are specific to the normal weight/style
    // First create a new font system to start fresh
    let mut font_system = FontSystem::new();
    
    // Reload our fonts
    let firamono_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("FiraMono-Medium.ttf"))
    );
    let inter_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("Inter-Regular.ttf"))
    );
    let notosans_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("NotoSans-Regular.ttf"))
    );
    let arabic_ids = font_system.db_mut().load_font_source(
        fontdb::Source::File(fonts_dir.join("NotoSansArabic.ttf"))
    );
    
    // Get the font IDs again
    let firamono_id = firamono_ids.first().copied().unwrap_or_else(|| {
        println!("Failed to load FiraMono font");
        // Use a fallback if failed
        fontdb::ID::dummy()
    });
    
    let inter_id = inter_ids.first().copied().unwrap_or_else(|| {
        println!("Failed to load Inter font");
        fontdb::ID::dummy()
    });
    
    let notosans_id = notosans_ids.first().copied().unwrap_or_else(|| {
        println!("Failed to load NotoSans font");
        fontdb::ID::dummy()
    });
    
    let arabic_id = arabic_ids.first().copied().unwrap_or_else(|| {
        println!("Failed to load Arabic font");
        fontdb::ID::dummy()
    });
    
    // a-f: FiraMono with Medium weight, Normal style - EXACT MATCH ONLY
    /*println!("  - a-f: {} ({}) with EXACT weight {:?}, style {:?}",
              firamono_id, firamono_postscript, firamono_weight, firamono_style);
    font_system.add_unicode_range_fallback_with_style(
        'a', 'f', firamono_id, Some(firamono_weight), Some(firamono_style)
    );
    
    // g-r: Inter with Regular weight, Normal style - EXACT MATCH ONLY
    println!("  - g-r: {} ({}) with EXACT weight {:?}, style {:?}", 
              inter_id, inter_postscript, inter_weight, inter_style);
    font_system.add_unicode_range_fallback_with_style(
        'g', 'r', inter_id, Some(inter_weight), Some(inter_style)
    );
    
    // s-z: NotoSans with Regular weight, Normal style - EXACT MATCH ONLY  
    println!("  - s-z: {} ({}) with EXACT weight {:?}, style {:?}", 
              notosans_id, notosans_postscript, notosans_weight, notosans_style);
    font_system.add_unicode_range_fallback_with_style(
        's', 'z', notosans_id, Some(notosans_weight), Some(notosans_style)
    );*/
    
    // Test with normal style and weight - should match
    /*println!("\n=== Test 3: With strict weight matching - normal weight ===");
    test_with_style_weight(
        &mut font_system, 
        "abcdefghijklmnopqrstuvwxyz", 
        &arabic_family, 
        Style::Normal, 
        Weight::NORMAL,
        &firamono_family, &inter_family, &notosans_family, 
        &firamono_postscript, &inter_postscript, &notosans_postscript
    )?;
    
    // Test with bold weight - should NOT match because our fallbacks require exact weight match
    println!("\n=== Test 4: With strict weight matching - bold weight (should NOT match) ===");
    test_with_style_weight(
        &mut font_system, 
        "abcdefghijklmnopqrstuvwxyz", 
        &arabic_family, 
        Style::Normal, 
        Weight::BOLD,
        &firamono_family, &inter_family, &notosans_family, 
        &firamono_postscript, &inter_postscript, &notosans_postscript
    )?;*/
    
    // Now add a fallback for bold weight
    println!("\nAdding a fallback for bold weight:");
    
    // A-Z: NotoSans with BOLD weight, Normal style
    println!("  - A-Z: {} ({}) with BOLD weight, Normal style", 
              notosans_id, notosans_postscript);
    font_system.add_unicode_range_fallback_with_style(
        'f', 'z', notosans_id, Some(Weight::BOLD), Some(Style::Normal)
    );
    
    // Test with bold weight on uppercase letters
    println!("\n=== Test 5: Bold uppercase should match specific bold fallback ===");
    test_with_style_weight(
        &mut font_system, 
        "abcdefghijklmnopqrstuvwxyz",
        &arabic_family, 
        Style::Normal, 
        Weight::BOLD,
        &firamono_family, &inter_family, &notosans_family, 
        &firamono_postscript, &inter_postscript, &notosans_postscript
    )?;
    
    Ok(())
}

// Helper function to test the text with a specific style and weight
fn test_with_style_weight(
    font_system: &mut FontSystem,
    text: &str,
    primary_family: &str,
    style: Style,
    weight: Weight,
    firamono_family: &str,
    inter_family: &str,
    notosans_family: &str,
    firamono_postscript: &str,
    inter_postscript: &str,
    notosans_postscript: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a buffer with Latin text
    let mut buffer = Buffer::new(font_system, Metrics::new(16.0, 24.0));
    
    // Create attributes with the specified style and weight
    println!("\nUsing '{}' as primary font with style {:?}, weight {:?}", 
             primary_family, style, weight);
    let attrs = Attrs::new()
        .family(Family::Name(primary_family))
        .style(style)
        .weight(weight);
    
    // Set the text with our attributes
    buffer.set_text(font_system, text, &attrs, Shaping::Advanced);
    
    // Set wrap to None to allow all text on one line
    buffer.set_wrap(font_system, Wrap::None);
    
    // Print the buffer for visual inspection
    println!("\nBuffer with text: {}", text);
    
    // Get information about which fonts were used for which characters
    let layout = buffer.layout_runs().next().unwrap();
    let mut char_to_font = HashMap::new();
    
    for glyph in layout.glyphs.iter() {
        let char_range = glyph.start..glyph.end;
        let character = &text[char_range.clone()];
        let font_id = glyph.font_id;
        
        // Get the font name and style/weight if available
        let font_info = if let Some(face) = font_system.db().face(font_id) {
            let name = if let Some((name, _)) = face.families.first() {
                format!("{} ({})", name, face.post_script_name)
            } else {
                face.post_script_name.clone()
            };
            
            // Include weight and style information
            let weight_str = match face.weight {
                Weight::NORMAL => "Regular",
                Weight::BOLD => "Bold",
                _ => "Other weight",
            };
            
            let style_str = match face.style {
                Style::Normal => "Normal",
                Style::Italic => "Italic",
                Style::Oblique => "Oblique",
            };
            
            format!("{} - {} {}", name, weight_str, style_str)
        } else {
            "unknown".to_string()
        };
        
        char_to_font.insert(character.to_string(), font_info);
    }
    
    // Create a function to determine the expected font for each character
    let get_expected_font = |c: char| -> String {
        // For lowercase a-f, expected font is FiraMono
        if ('a'..='f').contains(&c) {
            format!("{} ({}) - Regular Normal", firamono_family, firamono_postscript)
        } 
        // For lowercase g-r, expected font is Inter
        else if ('g'..='r').contains(&c) {
            format!("{} ({}) - Regular Normal", inter_family, inter_postscript)
        } 
        // For lowercase s-z, expected font is NotoSans
        else if ('s'..='z').contains(&c) {
            format!("{} ({}) - Regular Normal", notosans_family, notosans_postscript)
        }
        // For uppercase A-Z with bold weight, expected font is NotoSans Bold
        else if ('A'..='Z').contains(&c) && weight == Weight::BOLD {
            format!("{} ({}) - Bold Normal", notosans_family, notosans_postscript)
        }
        // No specific fallback configured
        else {
            "(none expected)".to_string()
        }
    };
    
    // Display used font and expected font for each character
    println!("\nFont usage by character:");
    for c in text.chars() {
        let c_str = c.to_string();
        let used_font = match char_to_font.get(&c_str) {
            Some(font_info) => font_info,
            None => "not found"
        };
        let expected_font = get_expected_font(c);
        
        // Check if this is a match with expected font
        let match_status = if used_font.contains(&expected_font) {
            "✓ MATCH"
        } else if expected_font == "(none expected)" {
            "- No expectation"
        } else {
            "✗ NO MATCH"
        };
        
        println!("'{}' -> Used: {}, Expected: {} {}", c, used_font, expected_font, match_status);
    }
    
    Ok(())
} 