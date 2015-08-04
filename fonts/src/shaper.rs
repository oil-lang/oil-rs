/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * NOTE:
 * This code is a modification of the file located at:
 *     https://github.com/servo/servo/blob/master/components/gfx/text/shaping/harfbuzz.rs
 */

use harfbuzz::{HB_MEMORY_MODE_READONLY, HB_DIRECTION_LTR, HB_DIRECTION_RTL};
use harfbuzz::{RUST_hb_blob_create, RUST_hb_face_create_for_tables};
use harfbuzz::{hb_blob_t};
use harfbuzz::{hb_bool_t};
use harfbuzz::{RUST_hb_buffer_add_utf8};
use harfbuzz::{RUST_hb_buffer_destroy};
use harfbuzz::{RUST_hb_buffer_get_glyph_positions};
use harfbuzz::{RUST_hb_buffer_get_length};
use harfbuzz::{RUST_hb_buffer_set_direction};
use harfbuzz::{RUST_hb_face_destroy};
use harfbuzz::{hb_face_t, hb_font_t};
use harfbuzz::{hb_feature_t};
use harfbuzz::{RUST_hb_font_create};
use harfbuzz::{RUST_hb_font_destroy, RUST_hb_buffer_create};
use harfbuzz::{RUST_hb_font_funcs_create};
use harfbuzz::{RUST_hb_font_funcs_destroy};
use harfbuzz::{RUST_hb_font_funcs_set_glyph_func};
use harfbuzz::{RUST_hb_font_funcs_set_glyph_h_advance_func};
use harfbuzz::{RUST_hb_font_funcs_set_glyph_h_kerning_func};
use harfbuzz::{hb_font_funcs_t, hb_buffer_t, hb_codepoint_t};
use harfbuzz::{RUST_hb_font_set_funcs};
use harfbuzz::{RUST_hb_font_set_ppem};
use harfbuzz::{RUST_hb_font_set_scale};
use harfbuzz::{hb_glyph_info_t};
use harfbuzz::{hb_glyph_position_t};
use harfbuzz::{hb_position_t, hb_tag_t};
use harfbuzz::{RUST_hb_shape, RUST_hb_buffer_get_glyph_infos};
use libc::{c_uint, c_int, c_void, c_char};
use std::char;
use std::mem;
use std::cmp;
use std::ptr;

use util::float_to_fixed;
use glyph::GlyphId;
use glyph::GlyphStore;
use font::FontTableTag;
use font::{RTL_FLAG, IGNORE_LIGATURES_SHAPING_FLAG, DISABLE_KERNING_SHAPING_FLAG};

use font::{Font, ShapingOptions};

struct FontAndShapingOptions {
    font: *mut Font,
    options: ShapingOptions,
}

pub struct Shaper {
    hb_face: *mut hb_face_t,
    hb_font: *mut hb_font_t,
    hb_funcs: *mut hb_font_funcs_t,
    font_and_shaping_options: Box<FontAndShapingOptions>,
}

impl Drop for Shaper {
    fn drop(&mut self) {
        unsafe {
            assert!(!self.hb_face.is_null());
            RUST_hb_face_destroy(self.hb_face);

            assert!(!self.hb_font.is_null());
            RUST_hb_font_destroy(self.hb_font);

            assert!(!self.hb_funcs.is_null());
            RUST_hb_font_funcs_destroy(self.hb_funcs);
        }
    }
}

impl Shaper {
    pub fn new(font: &mut Font, options: &ShapingOptions) -> Shaper {
        unsafe {
            let mut font_and_shaping_options = Box::new(FontAndShapingOptions {
                font: font,
                options: *options,
            });
            let hb_face: *mut hb_face_t =
                RUST_hb_face_create_for_tables(get_font_table_func,
                                          (&mut *font_and_shaping_options)
                                            as *mut FontAndShapingOptions
                                            as *mut c_void,
                                          None);
            let hb_font: *mut hb_font_t = RUST_hb_font_create(hb_face);

            // Set points-per-em. if zero, performs no hinting in that direction.
            let pt_size = font.actual_pt_size.to_f64_px();
            RUST_hb_font_set_ppem(hb_font, pt_size as c_uint, pt_size as c_uint);

            // Set scaling. Note that this takes 16.16 fixed point.
            RUST_hb_font_set_scale(hb_font,
                                   Shaper::float_to_fixed(pt_size) as c_int,
                                   Shaper::float_to_fixed(pt_size) as c_int);

            // configure static function callbacks.
            // NB. This funcs structure could be reused globally, as it never changes.
            let hb_funcs: *mut hb_font_funcs_t = RUST_hb_font_funcs_create();
            RUST_hb_font_funcs_set_glyph_func(hb_funcs, glyph_func, ptr::null_mut(), None);
            RUST_hb_font_funcs_set_glyph_h_advance_func(hb_funcs, glyph_h_advance_func, ptr::null_mut(), None);
            RUST_hb_font_funcs_set_glyph_h_kerning_func(hb_funcs, glyph_h_kerning_func, ptr::null_mut(), ptr::null_mut());
            RUST_hb_font_set_funcs(hb_font, hb_funcs, font as *mut Font as *mut c_void, None);

            Shaper {
                hb_face: hb_face,
                hb_font: hb_font,
                hb_funcs: hb_funcs,
                font_and_shaping_options: font_and_shaping_options,
            }
        }
    }

    /// Calculate the layout metrics associated with the given text when painted in a specific
    /// font.
    pub fn shape_text(&self, text: &str, options: &ShapingOptions, glyphs: &mut GlyphStore) {
        unsafe {
            let hb_buffer: *mut hb_buffer_t = RUST_hb_buffer_create();
            RUST_hb_buffer_set_direction(hb_buffer, if options.flags.contains(RTL_FLAG) {
                HB_DIRECTION_RTL
            } else {
                HB_DIRECTION_LTR
            });

            RUST_hb_buffer_add_utf8(hb_buffer,
                                    text.as_ptr() as *const c_char,
                                    text.len() as c_int,
                                    0,
                                    text.len() as c_int);

            let mut features = Vec::new();
            if options.flags.contains(IGNORE_LIGATURES_SHAPING_FLAG) {
                features.push(hb_feature_t {
                    _tag: LIGA,
                    _value: 0,
                    _start: 0,
                    _end: RUST_hb_buffer_get_length(hb_buffer),
                })
            }
            if options.flags.contains(DISABLE_KERNING_SHAPING_FLAG) {
                features.push(hb_feature_t {
                    _tag: KERN,
                    _value: 0,
                    _start: 0,
                    _end: RUST_hb_buffer_get_length(hb_buffer),
                })
            }

            RUST_hb_shape(self.hb_font, hb_buffer, features.as_mut_ptr(), features.len() as u32);
            self.save_glyph_results(text, options, glyphs, hb_buffer);
            RUST_hb_buffer_destroy(hb_buffer);
        }
    }

    pub fn set_options(&mut self, options: &ShapingOptions) {
        self.font_and_shaping_options.options = *options
    }

    fn save_glyph_results(&self,
                        text: &str,
                        options: &ShapingOptions,
                        glyphs: &mut GlyphStore,
                        buffer: *mut hb_buffer_t) {
        // TODO(Nemikolh) Cf:
        // https://github.com/servo/servo/blob/master/components/gfx/text/shaping/harfbuzz.rs#L259-L504
        panic!();
    }

    fn float_to_fixed(f: f64) -> i32 {
        float_to_fixed(16, f)
    }
}




macro_rules! hb_tag {
    ($t1:expr, $t2:expr, $t3:expr, $t4:expr) => (
        (($t1 as u32) << 24) | (($t2 as u32) << 16) | (($t3 as u32) << 8) | ($t4 as u32)
    );
}


static KERN: u32 = hb_tag!('k', 'e', 'r', 'n');
static LIGA: u32 = hb_tag!('l', 'i', 'g', 'a');


/// Callbacks from Harfbuzz when font map and glyph advance lookup needed.
extern fn glyph_func(_: *mut hb_font_t,
                     font_data: *mut c_void,
                     unicode: hb_codepoint_t,
                     _: hb_codepoint_t,
                     glyph: *mut hb_codepoint_t,
                     _: *mut c_void)
                  -> hb_bool_t {
    let font: *const Font = font_data as *const Font;
    assert!(!font.is_null());

    unsafe {
        match (*font).glyph_index(char::from_u32(unicode).unwrap()) {
            Some(g) => {
                *glyph = g as hb_codepoint_t;
                true as hb_bool_t
            }
            None => false as hb_bool_t
        }
    }
}

extern fn glyph_h_advance_func(_: *mut hb_font_t,
                               font_data: *mut c_void,
                               glyph: hb_codepoint_t,
                               _: *mut c_void)
                            -> hb_position_t {
    let font: *mut Font = font_data as *mut Font;
    assert!(!font.is_null());

    unsafe {
        let advance = (*font).glyph_h_advance(glyph as GlyphId);
        Shaper::float_to_fixed(advance)
    }
}

extern fn glyph_h_kerning_func(_: *mut hb_font_t,
                               font_data: *mut c_void,
                               first_glyph: hb_codepoint_t,
                               second_glyph: hb_codepoint_t,
                               _: *mut c_void)
                            -> hb_position_t {
    let font: *mut Font = font_data as *mut Font;
    assert!(!font.is_null());

    unsafe {
        let advance = (*font).glyph_h_kerning(first_glyph as GlyphId, second_glyph as GlyphId);
        Shaper::float_to_fixed(advance)
    }
}

// Callback to get a font table out of a font.
extern fn get_font_table_func(_: *mut hb_face_t,
                              tag: hb_tag_t,
                              user_data: *mut c_void)
                              -> *mut hb_blob_t {
    unsafe {
        // NB: These asserts have security implications.
        let font_and_shaping_options: *const FontAndShapingOptions =
            user_data as *const FontAndShapingOptions;
        assert!(!font_and_shaping_options.is_null());
        assert!(!(*font_and_shaping_options).font.is_null());

        // NOTE(Nemikolh): The code below is specific for macos CoreText library.
        // For now, we will only use freetype, so this code won't be used.
        // TODO(Issue #197): reuse font table data, which will change the unsound trickery here.
        // match (*(*font_and_shaping_options).font).get_table_for_tag(tag as FontTableTag) {
        //     None => ptr::null_mut(),
        //     Some(ref font_table) => {
        //         let skinny_font_table_ptr: *const FontTable = font_table;   // private context
        //
        //         let mut blob: *mut hb_blob_t = ptr::null_mut();
        //         (*skinny_font_table_ptr).with_buffer(|buf: *const u8, len: usize| {
        //             // HarfBuzz calls `destroy_blob_func` when the buffer is no longer needed.
        //             blob = RUST_hb_blob_create(buf as *const c_char,
        //                                        len as c_uint,
        //                                        HB_MEMORY_MODE_READONLY,
        //                                        mem::transmute(skinny_font_table_ptr),
        //                                        destroy_blob_func);
        //         });
        //
        //         assert!(!blob.is_null());
        //         blob
        //     }
        // }
        ptr::null_mut()
    }
}

extern fn destroy_blob_func(_: *mut c_void) {}
