use smallvec::{SmallVec8};
use std::u16;
use util::Au;
use euclid::Point2D;
use std::vec::Vec;
use std::iter::FromIterator;

/// Stores the glyph data belonging to a text run.
///
/// Simple glyphs are stored inline in the `entry_buffer`, detailed glyphs are
/// stored as pointers into the `detail_store`.
///
/// ~~~ignore
/// +- GlyphStore --------------------------------+
/// |               +---+---+---+---+---+---+---+ |
/// | entry_buffer: |   | s |   | s |   | s | s | |  d = detailed
/// |               +-|-+---+-|-+---+-|-+---+---+ |  s = simple
/// |                 |       |       |           |
/// |                 |   +---+-------+           |
/// |                 |   |                       |
/// |               +-V-+-V-+                     |
/// | detail_store: | d | d |                     |
/// |               +---+---+                     |
/// +---------------------------------------------+
/// ~~~
#[derive(Clone)]
pub struct GlyphStore {
    /// A buffer of glyphs within the text run, in the order in which they
    /// appear in the input text
    entry_buffer: SmallVec8<GlyphEntry>,
    /// A store of the detailed glyph data. Detailed glyphs contained in the
    /// `entry_buffer` point to locations in this data structure.
    detail_store: DetailedGlyphStore,

    is_whitespace: bool,
}

// Manages the lookup table for detailed glyphs. Sorting is deferred
// until a lookup is actually performed; this matches the expected
// usage pattern of setting/appending all the detailed glyphs, and
// then querying without setting.
#[derive(Clone)]
struct DetailedGlyphStore {
    detail_buffer: SmallVec8<DetailedGlyph>,
    detail_lookup: SmallVec8<DetailedGlyphRecord>,
    lookup_is_sorted: bool,
}

/// GlyphEntry is a port of Gecko's CompressedGlyph scheme for storing glyph data compactly.
///
/// In the common case (reasonable glyph advances, no offsets from the font em-box, and one glyph
/// per character), we pack glyph advance, glyph id, and some flags into a single u32.
///
/// In the uncommon case (multiple glyphs per unicode character, large glyph index/advance, or
/// glyph offsets), we pack the glyph count into GlyphEntry, and store the other glyph information
/// in DetailedGlyphStore.
#[derive(Clone, Debug, Copy)]
struct GlyphEntry {
    value: u32,
}

pub type GlyphId = u32;

/// Stores data for a detailed glyph, in the case that several glyphs
/// correspond to one character, or the glyph's data couldn't be packed.
#[derive(Clone, Debug, Copy)]
struct DetailedGlyph {
    id: GlyphId,
    // glyph's advance, in the text's direction (LTR or RTL)
    advance: Au,
    // glyph's offset from the font's em-box (from top-left)
    offset: Point2D<Au>,
}

#[derive(PartialEq, Clone, Eq, Debug, Copy)]
struct DetailedGlyphRecord {
    // source string offset/GlyphEntry offset in the TextRun
    entry_offset: isize,
    // offset into the detailed glyphs buffer
    detail_offset: usize,
}


static FLAG_CHAR_IS_SPACE: u32      = 0x10000000;
// These two bits store some BREAK_TYPE_* flags
static FLAG_CAN_BREAK_MASK: u32     = 0x60000000;
static FLAG_CAN_BREAK_SHIFT: u32    = 29;
static FLAG_IS_SIMPLE_GLYPH: u32    = 0x80000000;

// glyph advance; in Au's.
static GLYPH_ADVANCE_MASK: u32      = 0x0FFF0000;
static GLYPH_ADVANCE_SHIFT: u32     = 16;
static GLYPH_ID_MASK: u32           = 0x0000FFFF;

// Non-simple glyphs (more than one glyph per char; missing glyph,
// newline, tab, large advance, or nonzero x/y offsets) may have one
// or more detailed glyphs associated with them. They are stored in a
// side array so that there is a 1:1 mapping of GlyphEntry to
// unicode char.

// The number of detailed glyphs for this char. If the char couldn't
// be mapped to a glyph (!FLAG_NOT_MISSING), then this actually holds
// the UTF8 code point instead.
static GLYPH_COUNT_MASK:              u32 = 0x00FFFF00;
static GLYPH_COUNT_SHIFT:             u32 = 8;
// N.B. following Gecko, these are all inverted so that a lot of
// missing chars can be memset with zeros in one fell swoop.
static FLAG_NOT_MISSING:              u32 = 0x00000001;
static FLAG_NOT_CLUSTER_START:        u32 = 0x00000002;
static FLAG_NOT_LIGATURE_GROUP_START: u32 = 0x00000004;

static FLAG_CHAR_IS_TAB:              u32 = 0x00000008;
static FLAG_CHAR_IS_NEWLINE:          u32 = 0x00000010;

#[derive(PartialEq, Copy, Clone)]
pub enum BreakType {
    None,
    Normal,
    Hyphen,
}


static BREAK_TYPE_NONE:   u8 = 0x0;
static BREAK_TYPE_NORMAL: u8 = 0x1;
static BREAK_TYPE_HYPHEN: u8 = 0x2;

fn break_flag_to_enum(flag: u8) -> BreakType {
    if (flag & BREAK_TYPE_NORMAL) != 0 {
        BreakType::Normal
    } else if (flag & BREAK_TYPE_HYPHEN) != 0 {
        BreakType::Hyphen
    } else {
        BreakType::None
    }
}

fn break_enum_to_flag(e: BreakType) -> u8 {
    match e {
        BreakType::None   => BREAK_TYPE_NONE,
        BreakType::Normal => BREAK_TYPE_NORMAL,
        BreakType::Hyphen => BREAK_TYPE_HYPHEN,
    }
}

impl<'a> GlyphStore {
    /// Initializes the glyph store, but doesn't actually shape anything.
    ///
    /// Use the `add_*` methods to store glyph data.
    pub fn new(length: usize, is_whitespace: bool) -> GlyphStore {
        assert!(length > 0);
        GlyphStore {
            entry_buffer: SmallVec8::from_iter(
                vec![GlyphEntry::initial(); length].into_iter()),
            detail_store: DetailedGlyphStore::new(),
            is_whitespace: is_whitespace,
        }
    }
}

impl<'a> DetailedGlyphStore {
    fn new() -> DetailedGlyphStore {
        DetailedGlyphStore {
            detail_buffer: SmallVec8::from_iter(vec!().into_iter()),
            detail_lookup: SmallVec8::from_iter(vec!().into_iter()),
            lookup_is_sorted: false,
        }
    }
}

// Getters and setters for GlyphEntry. Setter methods are functional,
// because GlyphEntry is immutable and only a u32 in size.
impl GlyphEntry {

    fn new(value: u32) -> GlyphEntry {
        GlyphEntry {
            value: value,
        }
    }

    fn initial() -> GlyphEntry {
        GlyphEntry::new(0)
    }

    // Creates a GlyphEntry for the common case
    fn simple(id: GlyphId, advance: Au) -> GlyphEntry {
        assert!(is_simple_glyph_id(id));
        assert!(is_simple_advance(advance));

        let id_mask = id as u32;
        let Au(advance) = advance;
        let advance_mask = (advance as u32) << GLYPH_ADVANCE_SHIFT;

        GlyphEntry::new(id_mask | advance_mask | FLAG_IS_SIMPLE_GLYPH)
    }

    // Create a GlyphEntry for uncommon case; should be accompanied by
    // initialization of the actual DetailedGlyph data in DetailedGlyphStore
    fn complex(starts_cluster: bool, starts_ligature: bool, glyph_count: usize) -> GlyphEntry {
        assert!(glyph_count <= u16::MAX as usize);

        debug!("creating complex glyph entry: starts_cluster={}, starts_ligature={}, \
                glyph_count={}",
               starts_cluster,
               starts_ligature,
               glyph_count);

        let mut val = FLAG_NOT_MISSING;

        if !starts_cluster {
            val |= FLAG_NOT_CLUSTER_START;
        }
        if !starts_ligature {
            val |= FLAG_NOT_LIGATURE_GROUP_START;
        }
        val |= (glyph_count as u32) << GLYPH_COUNT_SHIFT;

        GlyphEntry::new(val)
    }

    /// Create a GlyphEntry for the case where glyphs couldn't be found for the specified
    /// character.
    fn missing(glyph_count: usize) -> GlyphEntry {
        assert!(glyph_count <= u16::MAX as usize);

        GlyphEntry::new((glyph_count as u32) << GLYPH_COUNT_SHIFT)
    }

    // getter methods
    #[inline(always)]
    fn advance(&self) -> Au {
        Au(((self.value & GLYPH_ADVANCE_MASK) >> GLYPH_ADVANCE_SHIFT) as i32)
    }

    fn id(&self) -> GlyphId {
        self.value & GLYPH_ID_MASK
    }

    fn is_ligature_start(&self) -> bool {
        self.has_flag(!FLAG_NOT_LIGATURE_GROUP_START)
    }

    fn is_cluster_start(&self) -> bool {
        self.has_flag(!FLAG_NOT_CLUSTER_START)
    }

    // True if original char was normal (U+0020) space. Other chars may
    // map to space glyph, but this does not account for them.
    fn char_is_space(&self) -> bool {
        self.has_flag(FLAG_CHAR_IS_SPACE)
    }

    fn char_is_tab(&self) -> bool {
        !self.is_simple() && self.has_flag(FLAG_CHAR_IS_TAB)
    }

    fn char_is_newline(&self) -> bool {
        !self.is_simple() && self.has_flag(FLAG_CHAR_IS_NEWLINE)
    }

    fn can_break_before(&self) -> BreakType {
        let flag = ((self.value & FLAG_CAN_BREAK_MASK) >> FLAG_CAN_BREAK_SHIFT) as u8;
        break_flag_to_enum(flag)
    }

    // setter methods
    #[inline(always)]
    fn set_char_is_space(&self) -> GlyphEntry {
        GlyphEntry::new(self.value | FLAG_CHAR_IS_SPACE)
    }

    #[inline(always)]
    fn set_char_is_tab(&self) -> GlyphEntry {
        assert!(!self.is_simple());
        GlyphEntry::new(self.value | FLAG_CHAR_IS_TAB)
    }

    #[inline(always)]
    fn set_char_is_newline(&self) -> GlyphEntry {
        assert!(!self.is_simple());
        GlyphEntry::new(self.value | FLAG_CHAR_IS_NEWLINE)
    }

    #[inline(always)]
    fn set_can_break_before(&self, e: BreakType) -> GlyphEntry {
        let flag = (break_enum_to_flag(e) as u32) << FLAG_CAN_BREAK_SHIFT;
        GlyphEntry::new(self.value | flag)
    }

    // helper methods

    fn glyph_count(&self) -> u16 {
        assert!(!self.is_simple());
        ((self.value & GLYPH_COUNT_MASK) >> GLYPH_COUNT_SHIFT) as u16
    }

    #[inline(always)]
    fn is_simple(&self) -> bool {
        self.has_flag(FLAG_IS_SIMPLE_GLYPH)
    }

    #[inline(always)]
    fn has_flag(&self, flag: u32) -> bool {
        (self.value & flag) != 0
    }

    #[inline(always)]
    fn adapt_character_flags_of_entry(&self, other: GlyphEntry) -> GlyphEntry {
        GlyphEntry { value: self.value | other.value }
    }
}


// ======================================== //
//                  HELPERS                 //
// ======================================== //


fn is_simple_glyph_id(id: GlyphId) -> bool {
    ((id as u32) & GLYPH_ID_MASK) == id
}

fn is_simple_advance(advance: Au) -> bool {
    advance >= Au(0) && {
        let unsigned_au = advance.0 as u32;
        (unsigned_au & (GLYPH_ADVANCE_MASK >> GLYPH_ADVANCE_SHIFT)) == unsigned_au
    }
}
