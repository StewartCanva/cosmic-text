use crate::{Attrs, Font, FontMatchAttrs, HashMap, ShapeBuffer};
use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use fontdb::Family;
use core::fmt;
use core::ops::{Deref, DerefMut};

// re-export fontdb and rustybuzz
pub use fontdb;
pub use rustybuzz;

use super::fallback::{Fallback, Fallbacks, MonospaceFallbackInfo, PlatformFallback, UnicodeRangeFallbacks};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FontMatchKey {
    pub(crate) font_weight_diff: u16,
    pub(crate) font_weight: u16,
    pub(crate) id: fontdb::ID,
}

struct FontCachedCodepointSupportInfo {
    supported: Vec<u32>,
    not_supported: Vec<u32>,
}

impl FontCachedCodepointSupportInfo {
    const SUPPORTED_MAX_SZ: usize = 512;
    const NOT_SUPPORTED_MAX_SZ: usize = 1024;

    fn new() -> Self {
        Self {
            supported: Vec::with_capacity(Self::SUPPORTED_MAX_SZ),
            not_supported: Vec::with_capacity(Self::NOT_SUPPORTED_MAX_SZ),
        }
    }

    #[inline(always)]
    fn unknown_has_codepoint(
        &mut self,
        font_codepoints: &[u32],
        codepoint: u32,
        supported_insert_pos: usize,
        not_supported_insert_pos: usize,
    ) -> bool {
        let ret = font_codepoints.contains(&codepoint);
        if ret {
            // don't bother inserting if we are going to truncate the entry away
            if supported_insert_pos != Self::SUPPORTED_MAX_SZ {
                self.supported.insert(supported_insert_pos, codepoint);
                self.supported.truncate(Self::SUPPORTED_MAX_SZ);
            }
        } else {
            // don't bother inserting if we are going to truncate the entry away
            if not_supported_insert_pos != Self::NOT_SUPPORTED_MAX_SZ {
                self.not_supported
                    .insert(not_supported_insert_pos, codepoint);
                self.not_supported.truncate(Self::NOT_SUPPORTED_MAX_SZ);
            }
        }
        ret
    }

    #[inline(always)]
    fn has_codepoint(&mut self, font_codepoints: &[u32], codepoint: u32) -> bool {
        match self.supported.binary_search(&codepoint) {
            Ok(_) => true,
            Err(supported_insert_pos) => match self.not_supported.binary_search(&codepoint) {
                Ok(_) => false,
                Err(not_supported_insert_pos) => self.unknown_has_codepoint(
                    font_codepoints,
                    codepoint,
                    supported_insert_pos,
                    not_supported_insert_pos,
                ),
            },
        }
    }
}

/// Access to the system fonts.
pub struct FontSystem {
    /// The locale of the system.
    locale: String,

    /// The underlying font database.
    db: fontdb::Database,

    /// Cache for loaded fonts from the database.
    font_cache: HashMap<fontdb::ID, Option<Arc<Font>>>,

    /// Sorted unique ID's of all Monospace fonts in DB
    monospace_font_ids: Vec<fontdb::ID>,

    /// Sorted unique ID's of all Monospace fonts in DB per script.
    /// A font may support multiple scripts of course, so the same ID
    /// may appear in multiple map value vecs.
    per_script_monospace_font_ids: HashMap<[u8; 4], Vec<fontdb::ID>>,

    /// Cache for font codepoint support info
    font_codepoint_support_info_cache: HashMap<fontdb::ID, FontCachedCodepointSupportInfo>,

    /// Cache for font matches.
    font_matches_cache: HashMap<FontMatchAttrs, Arc<Vec<FontMatchKey>>>,

    /// Scratch buffer for shaping and laying out.
    pub(crate) shape_buffer: ShapeBuffer,

    /// Buffer for use in `FontFallbackIter`.
    pub(crate) monospace_fallbacks_buffer: BTreeSet<MonospaceFallbackInfo>,

    /// Cache for shaped runs
    #[cfg(feature = "shape-run-cache")]
    pub shape_run_cache: crate::ShapeRunCache,

    /// List of fallbacks
    pub(crate) dyn_fallback: Box<dyn Fallback>,

    /// List of fallbacks
    pub(crate) fallbacks: Fallbacks,

    /// Unicode range-specific fallbacks
    pub(crate) unicode_range_fallbacks: UnicodeRangeFallbacks,
}

impl fmt::Debug for FontSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FontSystem")
            .field("locale", &self.locale)
            .field("db", &self.db)
            .finish()
    }
}

impl FontSystem {
    const FONT_MATCHES_CACHE_SIZE_LIMIT: usize = 256;
    /// Create a new [`FontSystem`], that allows access to any installed system fonts
    ///
    /// # Timing
    ///
    /// This function takes some time to run. On the release build, it can take up to a second,
    /// while debug builds can take up to ten times longer. For this reason, it should only be
    /// called once, and the resulting [`FontSystem`] should be shared.
    pub fn new() -> Self {
        Self::new_with_fonts(core::iter::empty())
    }

    /// Create a new [`FontSystem`] with a pre-specified set of fonts.
    pub fn new_with_fonts(fonts: impl IntoIterator<Item = fontdb::Source>) -> Self {
        let locale = Self::get_locale();
        log::debug!("Locale: {}", locale);

        let mut db = fontdb::Database::new();

        Self::load_fonts(&mut db, fonts.into_iter());

        //TODO: configurable default fonts
        db.set_monospace_family("Noto Sans Mono");
        db.set_sans_serif_family("Open Sans");
        db.set_serif_family("DejaVu Serif");

        Self::new_with_locale_and_db_and_fallback(locale, db, PlatformFallback)
    }

    /// Create a new [`FontSystem`] with a pre-specified locale, font database and font fallback list.
    pub fn new_with_locale_and_db_and_fallback<F: Fallback + 'static>(
        locale: String,
        db: fontdb::Database,
        impl_fallback: F,
    ) -> Self {
        let mut monospace_font_ids = db
            .faces()
            .filter(|face_info| {
                face_info.monospaced && !face_info.post_script_name.contains("Emoji")
            })
            .map(|face_info| face_info.id)
            .collect::<Vec<_>>();
        monospace_font_ids.sort();

        let mut per_script_monospace_font_ids: HashMap<[u8; 4], BTreeSet<fontdb::ID>> =
            HashMap::default();

        if cfg!(feature = "monospace_fallback") {
            monospace_font_ids.iter().for_each(|&id| {
                db.with_face_data(id, |font_data, face_index| {
                    let _ = ttf_parser::Face::parse(font_data, face_index).map(|face| {
                        face.tables()
                            .gpos
                            .into_iter()
                            .chain(face.tables().gsub)
                            .flat_map(|table| table.scripts)
                            .inspect(|script| {
                                per_script_monospace_font_ids
                                    .entry(script.tag.to_bytes())
                                    .or_default()
                                    .insert(id);
                            })
                    });
                });
            });
        }

        let per_script_monospace_font_ids = per_script_monospace_font_ids
            .into_iter()
            .map(|(k, v)| (k, Vec::from_iter(v)))
            .collect();

        let fallbacks = Fallbacks::new(&impl_fallback, &[], &locale);

        Self {
            locale,
            db,
            monospace_font_ids,
            per_script_monospace_font_ids,
            font_cache: Default::default(),
            font_matches_cache: Default::default(),
            font_codepoint_support_info_cache: Default::default(),
            monospace_fallbacks_buffer: BTreeSet::default(),
            #[cfg(feature = "shape-run-cache")]
            shape_run_cache: crate::ShapeRunCache::default(),
            shape_buffer: ShapeBuffer::default(),
            dyn_fallback: Box::new(impl_fallback),
            fallbacks,
            unicode_range_fallbacks: UnicodeRangeFallbacks::new(),
        }
    }

    /// Create a new [`FontSystem`] with a pre-specified locale and font database.
    pub fn new_with_locale_and_db(locale: String, db: fontdb::Database) -> Self {
        Self::new_with_locale_and_db_and_fallback(locale, db, PlatformFallback)
    }

    /// Get the locale.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Get the database.
    pub fn db(&self) -> &fontdb::Database {
        &self.db
    }

    /// Get a mutable reference to the database.
    pub fn db_mut(&mut self) -> &mut fontdb::Database {
        self.font_matches_cache.clear();
        &mut self.db
    }

    /// Consume this [`FontSystem`] and return the locale and database.
    pub fn into_locale_and_db(self) -> (String, fontdb::Database) {
        (self.locale, self.db)
    }

    /// Get a font by its ID.
    pub fn get_font(&mut self, id: fontdb::ID) -> Option<Arc<Font>> {
        self.font_cache
            .entry(id)
            .or_insert_with(|| {
                #[cfg(feature = "std")]
                unsafe {
                    self.db.make_shared_face_data(id);
                }
                match Font::new(&self.db, id) {
                    Some(font) => Some(Arc::new(font)),
                    None => {
                        log::warn!(
                            "failed to load font '{}'",
                            self.db.face(id)?.post_script_name
                        );
                        None
                    }
                }
            })
            .clone()
    }

    pub fn is_monospace(&self, id: fontdb::ID) -> bool {
        self.monospace_font_ids.binary_search(&id).is_ok()
    }

    pub fn get_monospace_ids_for_scripts(
        &self,
        scripts: impl Iterator<Item = [u8; 4]>,
    ) -> Vec<fontdb::ID> {
        let mut ret = scripts
            .filter_map(|script| self.per_script_monospace_font_ids.get(&script))
            .flat_map(|ids| ids.iter().copied())
            .collect::<Vec<_>>();
        ret.sort();
        ret.dedup();
        ret
    }

    #[inline(always)]
    pub fn get_font_supported_codepoints_in_word(
        &mut self,
        id: fontdb::ID,
        word: &str,
    ) -> Option<usize> {
        self.get_font(id).map(|font| {
            let code_points = font.unicode_codepoints();
            let cache = self
                .font_codepoint_support_info_cache
                .entry(id)
                .or_insert_with(FontCachedCodepointSupportInfo::new);
            word.chars()
                .filter(|ch| cache.has_codepoint(code_points, u32::from(*ch)))
                .count()
        })
    }

    pub fn get_font_matches(&mut self, attrs: &Attrs<'_>) -> Arc<Vec<FontMatchKey>> {
        // Clear the cache first if it reached the size limit
        if self.font_matches_cache.len() >= Self::FONT_MATCHES_CACHE_SIZE_LIMIT {
            log::trace!("clear font mache cache");
            self.font_matches_cache.clear();
        }

        self.font_matches_cache
            //TODO: do not create AttrsOwned unless entry does not already exist
            .entry(attrs.into())
            .or_insert_with(|| {
                #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                let now = std::time::Instant::now();

                let mut font_match_keys = self
                    .db
                    .faces()
                    .filter(|face| attrs.matches(face))
                    .map(|face| FontMatchKey {
                        font_weight_diff: attrs.weight.0.abs_diff(face.weight.0),
                        font_weight: face.weight.0,
                        id: face.id,
                    })
                    .collect::<Vec<_>>();

                // Sort so we get the keys with weight_offset=0 first
                font_match_keys.sort();

                #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
                {
                    let elapsed = now.elapsed();
                    log::debug!("font matches for {:?} in {:?}", attrs, elapsed);
                }

                Arc::new(font_match_keys)
            })
            .clone()
    }

    #[cfg(feature = "std")]
    fn get_locale() -> String {
        sys_locale::get_locale().unwrap_or_else(|| {
            log::warn!("failed to get system locale, falling back to en-US");
            String::from("en-US")
        })
    }

    #[cfg(not(feature = "std"))]
    fn get_locale() -> String {
        String::from("en-US")
    }

    #[cfg(feature = "std")]
    fn load_fonts(db: &mut fontdb::Database, fonts: impl Iterator<Item = fontdb::Source>) {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();

        db.load_system_fonts();

        for source in fonts {
            db.load_font_source(source);
        }

        #[cfg(not(target_arch = "wasm32"))]
        log::debug!(
            "Parsed {} font faces in {}ms.",
            db.len(),
            now.elapsed().as_millis()
        );
    }

    #[cfg(not(feature = "std"))]
    fn load_fonts(db: &mut fontdb::Database, fonts: impl Iterator<Item = fontdb::Source>) {
        for source in fonts {
            db.load_font_source(source);
        }
    }

    /// Add a Unicode range fallback
    pub fn add_unicode_range_fallback(&mut self, start: char, end: char, font_id: fontdb::ID) {
        self.unicode_range_fallbacks.add(start, end, font_id);
    }
    
    /// Add a Unicode range fallback with specific weight and style
    pub fn add_unicode_range_fallback_with_style(
        &mut self, 
        start: char, 
        end: char, 
        font_id: fontdb::ID,
        weight: Option<fontdb::Weight>,
        style: Option<fontdb::Style>
    ) {
        self.unicode_range_fallbacks.add_with_style(start, end, font_id, weight, style);
    }
    
    /// Get the fallback font ID for a specific character
    pub fn get_unicode_range_fallback_for_char(&mut self, c: char) -> Option<fontdb::ID> {
        self.unicode_range_fallbacks.find_for_char(c)
    }
    
    /// Get the fallback font ID for a specific character with weight and style matching
    pub fn get_unicode_range_fallback_for_char_with_style(
        &mut self, 
        c: char,
        weight: Option<fontdb::Weight>,
        style: Option<fontdb::Style>
    ) -> Option<fontdb::ID> {
        self.unicode_range_fallbacks.find_for_char_with_style(c, weight, style)
    }
    
    /// Check if any Unicode range fallbacks are defined
    pub fn has_unicode_range_fallbacks(&self) -> bool {
        !self.unicode_range_fallbacks.is_empty()
    }
    
    /// Process Unicode range fallbacks for missing character positions
    pub fn process_unicode_range_fallbacks(
        &mut self,
        glyphs: &mut Vec<crate::ShapeGlyph>,
        line: &str,
        attrs_list: &crate::AttrsList,
        start_run: usize,
        end_run: usize,
        span_rtl: bool,
        missing: &[usize],
    ) -> Vec<usize> {
        // Okay this makes it really complicated now.
        // In the case that we haven't shaped anything
        // we need to find one fo the 
        // Actually before I ruin this. Can I 


        
        // If no Unicode range fallbacks are defined, return quickly
        if !self.has_unicode_range_fallbacks() {
            return missing.to_vec();
        }
        
        // Get attributes for matching fonts
        let attrs = attrs_list.get_span(start_run);
        
        // Group missing positions by fallback font ID
        let mut font_id_to_positions: HashMap<fontdb::ID, Vec<usize>> = HashMap::default();
        let mut positions_without_fallback: Vec<usize> = Vec::new();
        
        // TODO: It's possible that missing is empty here 
        // Actually, missing are indices into grapheme clusters, does this even work?
        // Okay what actual information do we get from harfbuzz. Can we map the grapheme cluster
        // to the start of the character?
        // From harfbuzz. An index to the start of the grapheme cluster in the original string.
        // Thaat's not too bad.
        for &pos in missing {
            // Find the character at this position
            let char_offset = pos.saturating_sub(start_run);
            if char_offset >= line[start_run..end_run].len() {
                positions_without_fallback.push(pos);
                continue;
            }
            
            let c = match line[start_run..end_run][char_offset..].chars().next() {
                Some(c) => c,
                None => {
                    positions_without_fallback.push(pos);
                    continue;
                }
            };

            print!("COSMIC TEXT: {:?}, {:?}", attrs.weight, attrs.style);
            
            // Try to find a fallback font that matches the character, weight, and style
            if let Some(font_id) = self.get_unicode_range_fallback_for_char_with_style(
                c, 
                Some(attrs.weight), 
                Some(attrs.style)
            ) {
                font_id_to_positions.entry(font_id).or_default().push(pos);
            } 
            // If no match with specific weight and style, try just the character
            // Actually this will prevent us from using the correct fallback, when text changes weight after first shape.
            /*else if let Some(font_id) = self.get_unicode_range_fallback_for_char(c) {
                font_id_to_positions.entry(font_id).or_default().push(pos);
            } */
            else {
                positions_without_fallback.push(pos);
            }
        }
        
        // Process each fallback font ID and its positions
        let mut remaining_missing = positions_without_fallback;
        
        for (font_id, positions) in font_id_to_positions {
            if let Some(font) = self.get_font(font_id) {
                // Shape the entire run with this font
                let mut fb_glyphs = Vec::new();
                
                // Use a temporary buffer to shape with the fallback font
                let mut buffer = crate::ShapeBuffer::default();
                let (fb_missing, _) = crate::shape_fallback(
                    &mut buffer,
                    &mut fb_glyphs,
                    &font,
                    line,
                    attrs_list,
                    start_run,
                    end_run,
                    span_rtl,
                );
                
                // Process positions that should be covered by this font
                for pos in positions {
                    if !fb_missing.contains(&pos) {
                        // Find the corresponding glyph in fb_glyphs
                        for fb_glyph in &fb_glyphs {
                            if fb_glyph.start == pos {
                                // Add or replace the glyph in our results
                                let mut found = false;
                                for glyph in glyphs.iter_mut() {
                                    if glyph.start == pos {
                                        *glyph = fb_glyph.clone();
                                        found = true;
                                        break;
                                    }
                                }
                                if !found {
                                    glyphs.push(fb_glyph.clone());
                                }
                                break;
                            }
                        }
                    } else {
                        // This position wasn't covered by the font after all
                        remaining_missing.push(pos);
                    }
                }
            } else {
                // Font not found, keep positions in missing list
                remaining_missing.extend(positions);
            }
        }
        
        // Sort by character position
        glyphs.sort_by_key(|g| g.start);
        remaining_missing.sort();
        
        remaining_missing
    }
}

/// A value borrowed together with an [`FontSystem`]
#[derive(Debug)]
pub struct BorrowedWithFontSystem<'a, T> {
    pub(crate) inner: &'a mut T,
    pub(crate) font_system: &'a mut FontSystem,
}

impl<T> Deref for BorrowedWithFontSystem<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> DerefMut for BorrowedWithFontSystem<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
