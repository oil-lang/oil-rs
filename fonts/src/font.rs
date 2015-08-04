use util::Au;
use freetype::FontHandle;
use shaper::Shaper;
use glyph::GlyphId;
use util::cache::HashCache;
use std::sync::Arc;
use glyph::GlyphStore;

// Used to abstract over the shaper's choice of fixed int representation.
pub type FractionalPixel = f64;

pub type FontTableTag = u32;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FontTemplateDescriptor {
    italic: bool,
}

//#[derive(Clone, Debug, Deserialize, Serialize)]
#[derive(Clone, Debug)]
pub struct FontMetrics {
    pub underline_size:   Au,
    pub underline_offset: Au,
    pub strikeout_size:   Au,
    pub strikeout_offset: Au,
    pub leading:          Au,
    pub x_height:         Au,
    pub em_size:          Au,
    pub ascent:           Au,
    pub descent:          Au,
    pub max_advance:      Au,
    pub average_advance:  Au,
    pub line_gap:         Au,
}

pub struct Font {
    pub handle: FontHandle,
    pub metrics: FontMetrics,
    // pub variant: font_variant::T,
    pub descriptor: FontTemplateDescriptor,
    pub requested_pt_size: Au,
    pub actual_pt_size: Au,
    pub shaper: Option<Shaper>,
    pub shape_cache: HashCache<ShapeCacheEntry,Arc<GlyphStore>>,
    pub glyph_advance_cache: HashCache<u32,FractionalPixel>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ShapingOptions {
    /// Various flags.
    pub flags: ShapingFlags,
}

bitflags! {
    flags ShapingFlags: u8 {
        #[doc="Set if the text is entirely whitespace."]
        const IS_WHITESPACE_SHAPING_FLAG = 0x01,
        #[doc="Set if we are to ignore ligatures."]
        const IGNORE_LIGATURES_SHAPING_FLAG = 0x02,
        #[doc="Set if we are to disable kerning."]
        const DISABLE_KERNING_SHAPING_FLAG = 0x04,
        #[doc="Text direction is right-to-left."]
        const RTL_FLAG = 0x08,
    }
}

/// An entry in the shape cache.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ShapeCacheEntry {
    text: String,
    options: ShapingOptions,
}

impl Font {
    pub fn shape_text(&mut self, text: &str, options: &ShapingOptions) -> Arc<GlyphStore> {
        self.make_shaper(options);

        //FIXME: find the equivalent of Equiv and the old ShapeCacheEntryRef
        let shaper = &self.shaper;
        let lookup_key = ShapeCacheEntry {
            text: text.to_owned(),
            options: options.clone(),
        };
        if let Some(glyphs) = self.shape_cache.find(&lookup_key) {
            return glyphs.clone();
        }

        let mut glyphs = GlyphStore::new(text.chars().count(),
                                         options.flags.contains(IS_WHITESPACE_SHAPING_FLAG));
        shaper.as_ref().unwrap().shape_text(text, options, &mut glyphs);

        let glyphs = Arc::new(glyphs);
        self.shape_cache.insert(ShapeCacheEntry {
            text: text.to_owned(),
            options: *options,
        }, glyphs.clone());
        glyphs
    }

    fn make_shaper<'a>(&'a mut self, options: &ShapingOptions) -> &'a Shaper {
        // fast path: already created a shaper
        if let Some(ref mut shaper) = self.shaper {
            shaper.set_options(options);
            return shaper
        }

        let shaper = Shaper::new(self, options);
        self.shaper = Some(shaper);
        self.shaper.as_ref().unwrap()
    }

    #[inline]
    pub fn glyph_index(&self, codepoint: char) -> Option<GlyphId> {
        // let codepoint = match self.variant {
        //     font_variant::T::small_caps => codepoint.to_uppercase().next().unwrap(), //FIXME: #5938
        //     font_variant::T::normal => codepoint,
        // };
        self.handle.glyph_index(codepoint)
    }

    pub fn glyph_h_kerning(&mut self, first_glyph: GlyphId, second_glyph: GlyphId)
                           -> FractionalPixel {
        self.handle.glyph_h_kerning(first_glyph, second_glyph)
    }

    pub fn glyph_h_advance(&mut self, glyph: GlyphId) -> FractionalPixel {
        let handle = &self.handle;
        self.glyph_advance_cache.find_or_create(&glyph, |glyph| {
            match handle.glyph_h_advance(*glyph) {
                Some(adv) => adv,
                None => 10f64 as FractionalPixel // FIXME: Need fallback strategy
            }
        })
    }
}
