// SPDX-License-Identifier: GPL-2.0 OR MIT

//! DRM panic QR helpers.
//!
//! The QR encoding core in this file is adapted from Project Nayuki's
//! no-heap QR Code generator library under the MIT license:
//! <https://www.nayuki.io/page/qr-code-generator-library>.

#![allow(dead_code)]

use crate::{prelude::c_char, str::CStrExt as _};
use core::{convert::TryFrom, ffi::CStr, slice};

const QR_ECC: QrCodeEcc = QrCodeEcc::Low;
const MAX_QR_BUFFER_LEN: usize = Version::MAX.buffer_len();

pub fn max_data_size(version: u8, url_len: usize) -> usize {
    if !(Version::MIN.value()..=Version::MAX.value()).contains(&version) {
        return 0;
    }

    let version = Version::new(version);
    if url_len == 0 {
        max_raw_data_size_for(version)
    } else {
        max_url_data_size_for(version, url_len)
    }
}

pub fn generate_from_raw(
    url: *const c_char,
    data: *mut u8,
    data_len: usize,
    data_size: usize,
    tmp: *mut u8,
    tmp_size: usize,
) -> u8 {
    if data.is_null() || tmp.is_null() || data_len == 0 || data_len > data_size {
        return 0;
    }

    // SAFETY:
    // - The C caller owns `data..data+data_size` for the duration of this call.
    // - The C caller owns `tmp..tmp+tmp_size` for the duration of this call.
    // - The checked `data_len <= data_size` precondition keeps the payload slice in-bounds.
    let data = unsafe { slice::from_raw_parts_mut(data, data_size) };
    let tmp = unsafe { slice::from_raw_parts_mut(tmp, tmp_size) };

    if data.len() < MAX_QR_BUFFER_LEN || tmp.len() < MAX_QR_BUFFER_LEN {
        return 0;
    }

    let url = if url.is_null() {
        None
    } else {
        // SAFETY: The C side passes either NULL or a valid NUL-terminated string.
        Some(unsafe { CStr::from_char_ptr(url) })
    };

    let result = match url {
        Some(url) => generate_url(url.to_bytes(), data, data_len, tmp),
        None => generate_raw(data, data_len, tmp),
    };

    result.unwrap_or(0)
}

fn generate_raw(data: &mut [u8], data_len: usize, tmp: &mut [u8]) -> Option<u8> {
    let qr = QrCode::encode_binary(
        data,
        data_len,
        tmp,
        QR_ECC,
        Version::MIN,
        Version::MAX,
        None,
        false,
    )
    .ok()?;

    pack_bitmap(&qr, data)
}

fn generate_url(url: &[u8], data: &mut [u8], data_len: usize, tmp: &mut [u8]) -> Option<u8> {
    let segment_len = data_len.checked_mul(10)?.checked_add(7)? / 8;
    let total_len = segment_len.checked_add(MAX_QR_BUFFER_LEN)?;
    if tmp.len() < total_len {
        return None;
    }

    let (segment_buf, qrbuf_tail) = tmp.split_at_mut(segment_len);
    let qrbuf = &mut qrbuf_tail[..MAX_QR_BUFFER_LEN];

    let (data_codewords_len, version) = {
        let payload = &data[..data_len];
        let url_segment = QrSegment::make_bytes(url);
        let numeric_segment = make_numeric_byte_segment(payload, segment_buf)?;
        let (data_codewords_len, _ecl, version) = QrCode::encode_segments_to_codewords(
            &[url_segment, numeric_segment],
            qrbuf,
            QR_ECC,
            Version::MIN,
            Version::MAX,
            false,
        )
        .ok()?;

        (data_codewords_len, version)
    };

    let qr = QrCode::encode_codewords(
        qrbuf,
        data_codewords_len,
        &mut data[..MAX_QR_BUFFER_LEN],
        QR_ECC,
        version,
        None,
    );

    pack_bitmap(&qr, data)
}

fn make_numeric_byte_segment<'a>(payload: &[u8], buf: &'a mut [u8]) -> Option<QrSegment<'a>> {
    let needed = payload.len().checked_mul(10)?.checked_add(7)? / 8;
    let buf = buf.get_mut(..needed)?;
    let mut bb = BitBuffer::new(buf);

    for &byte in payload {
        bb.append_bits(u32::from(byte), 10);
    }

    Some(QrSegment::new(
        QrSegmentMode::Numeric,
        payload.len().checked_mul(3)?,
        bb.data,
        bb.length,
    ))
}

fn pack_bitmap(qr: &QrCode<'_>, out: &mut [u8]) -> Option<u8> {
    let width = usize::try_from(qr.size()).ok()?;
    let pitch = width.checked_add(7)? / 8;
    let image_len = pitch.checked_mul(width)?;
    let out = out.get_mut(..image_len)?;
    out.fill(0);

    for y in 0..width {
        for x in 0..width {
            // The C draw path paints set bits with the foreground color on top of
            // a background-filled QR rectangle, so we store light modules as 1 bits.
            if !qr.get_module(x as i32, y as i32) {
                let index = y * pitch + x / 8;
                out[index] |= 0x80u8 >> (x % 8);
            }
        }
    }

    u8::try_from(width).ok()
}

fn max_raw_data_size_for(version: Version) -> usize {
    let capacity_bits = QrCode::get_num_data_codewords(version, QR_ECC) * 8;
    let overhead_bits = 4 + usize::from(QrSegmentMode::Byte.num_char_count_bits(version));

    capacity_bits
        .checked_sub(overhead_bits)
        .map(|bits| bits / 8)
        .unwrap_or(0)
}

fn max_url_data_size_for(version: Version, url_len: usize) -> usize {
    let capacity_bits = QrCode::get_num_data_codewords(version, QR_ECC) * 8;
    let url_bits = match url_len.checked_mul(8) {
        Some(bits) => bits,
        None => return 0,
    };
    let overhead_bits = 4
        + usize::from(QrSegmentMode::Byte.num_char_count_bits(version))
        + 4
        + usize::from(QrSegmentMode::Numeric.num_char_count_bits(version));
    let required_bits = match url_bits.checked_add(overhead_bits) {
        Some(bits) => bits,
        None => return 0,
    };

    capacity_bits
        .checked_sub(required_bits)
        .map(|bits| bits / 10)
        .unwrap_or(0)
}

/*---- QrCode functionality ----*/

/// A QR Code symbol, which is a type of two-dimension barcode.
///
/// Invented by Denso Wave and described in the ISO/IEC 18004 standard.
///
/// Instances of this struct represent an immutable square grid of dark and light cells.
/// The impl provides static factory functions to create a QR Code from text or binary data.
/// The struct and impl cover the QR Code Model 2 specification, supporting all versions
/// (sizes) from 1 to 40, all 4 error correction levels, and 4 character encoding modes.
///
/// Ways to create a QR Code object:
///
/// - High level: Take the payload data and call `QrCode::encode_text()` or `QrCode::encode_binary()`.
/// - Mid level: Custom-make the list of segments and call
///   `QrCode::encode_segments_to_codewords()` and then `QrCode::encode_codewords()`.
/// - Low level: Custom-make the array of data codeword bytes (including segment
///   headers and final padding, excluding error correction codewords), supply the
///   appropriate version number, and call the `QrCode::encode_codewords()` constructor.
///
/// (Note that all ways require supplying the desired error correction level and various byte buffers.)
pub struct QrCode<'a> {
    size: &'a mut u8,
    modules: &'a mut [u8],
}

impl<'a> QrCode<'a> {
    pub fn encode_text<'b>(
        text: &str,
        tempbuffer: &'b mut [u8],
        mut outbuffer: &'a mut [u8],
        ecl: QrCodeEcc,
        minversion: Version,
        maxversion: Version,
        mask: Option<Mask>,
        boostecl: bool,
    ) -> Result<QrCode<'a>, DataTooLong> {
        let minlen: usize = outbuffer.len().min(tempbuffer.len());
        outbuffer = &mut outbuffer[..minlen];

        let textlen: usize = text.len();
        if textlen == 0 {
            let (datacodewordslen, ecl, version) = QrCode::encode_segments_to_codewords(
                &[],
                outbuffer,
                ecl,
                minversion,
                maxversion,
                boostecl,
            )?;
            return Ok(Self::encode_codewords(
                outbuffer,
                datacodewordslen,
                tempbuffer,
                ecl,
                version,
                mask,
            ));
        }

        use QrSegmentMode::*;
        let buflen: usize = outbuffer.len();
        let seg: QrSegment = if QrSegment::is_numeric(text)
            && QrSegment::calc_buffer_size(Numeric, textlen).map_or(false, |x| x <= buflen)
        {
            QrSegment::make_numeric(text, tempbuffer)
        } else if QrSegment::is_alphanumeric(text)
            && QrSegment::calc_buffer_size(Alphanumeric, textlen).map_or(false, |x| x <= buflen)
        {
            QrSegment::make_alphanumeric(text, tempbuffer)
        } else if QrSegment::calc_buffer_size(Byte, textlen).map_or(false, |x| x <= buflen) {
            QrSegment::make_bytes(text.as_bytes())
        } else {
            return Err(DataTooLong::SegmentTooLong);
        };
        let (datacodewordslen, ecl, version) = QrCode::encode_segments_to_codewords(
            &[seg],
            outbuffer,
            ecl,
            minversion,
            maxversion,
            boostecl,
        )?;
        Ok(Self::encode_codewords(
            outbuffer,
            datacodewordslen,
            tempbuffer,
            ecl,
            version,
            mask,
        ))
    }

    pub fn encode_binary<'b>(
        dataandtempbuffer: &'b mut [u8],
        datalen: usize,
        mut outbuffer: &'a mut [u8],
        ecl: QrCodeEcc,
        minversion: Version,
        maxversion: Version,
        mask: Option<Mask>,
        boostecl: bool,
    ) -> Result<QrCode<'a>, DataTooLong> {
        assert!(datalen <= dataandtempbuffer.len(), "Invalid data length");
        let minlen: usize = outbuffer.len().min(dataandtempbuffer.len());
        outbuffer = &mut outbuffer[..minlen];

        if QrSegment::calc_buffer_size(QrSegmentMode::Byte, datalen)
            .map_or(true, |x| x > outbuffer.len())
        {
            return Err(DataTooLong::SegmentTooLong);
        }
        let seg: QrSegment = QrSegment::make_bytes(&dataandtempbuffer[..datalen]);
        let (datacodewordslen, ecl, version) = QrCode::encode_segments_to_codewords(
            &[seg],
            outbuffer,
            ecl,
            minversion,
            maxversion,
            boostecl,
        )?;
        Ok(Self::encode_codewords(
            outbuffer,
            datacodewordslen,
            dataandtempbuffer,
            ecl,
            version,
            mask,
        ))
    }

    pub fn encode_segments_to_codewords(
        segs: &[QrSegment],
        outbuffer: &'a mut [u8],
        mut ecl: QrCodeEcc,
        minversion: Version,
        maxversion: Version,
        boostecl: bool,
    ) -> Result<(usize, QrCodeEcc, Version), DataTooLong> {
        assert!(minversion <= maxversion, "Invalid value");
        assert!(
            outbuffer.len() >= QrCode::get_num_data_codewords(maxversion, ecl),
            "Invalid buffer length"
        );

        let mut version: Version = minversion;
        let datausedbits: usize = loop {
            let datacapacitybits: usize = QrCode::get_num_data_codewords(version, ecl) * 8;
            let dataused: Option<usize> = QrSegment::get_total_bits(segs, version);
            if dataused.map_or(false, |n| n <= datacapacitybits) {
                break dataused.unwrap();
            } else if version >= maxversion {
                return Err(match dataused {
                    None => DataTooLong::SegmentTooLong,
                    Some(n) => DataTooLong::DataOverCapacity(n, datacapacitybits),
                });
            } else {
                version = Version::new(version.value() + 1);
            }
        };

        for &newecl in &[QrCodeEcc::Medium, QrCodeEcc::Quartile, QrCodeEcc::High] {
            if boostecl && datausedbits <= QrCode::get_num_data_codewords(version, newecl) * 8 {
                ecl = newecl;
            }
        }

        let datacapacitybits: usize = QrCode::get_num_data_codewords(version, ecl) * 8;
        let mut bb = BitBuffer::new(&mut outbuffer[..datacapacitybits / 8]);
        for seg in segs {
            bb.append_bits(seg.mode.mode_bits(), 4);
            bb.append_bits(
                u32::try_from(seg.numchars).unwrap(),
                seg.mode.num_char_count_bits(version),
            );
            for i in 0..seg.bitlength {
                let bit: u8 = (seg.data[i >> 3] >> (7 - (i & 7))) & 1;
                bb.append_bits(bit.into(), 1);
            }
        }
        debug_assert_eq!(bb.length, datausedbits);

        let numzerobits: usize = core::cmp::min(4, datacapacitybits - bb.length);
        bb.append_bits(0, u8::try_from(numzerobits).unwrap());
        let numzerobits: usize = bb.length.wrapping_neg() & 7;
        bb.append_bits(0, u8::try_from(numzerobits).unwrap());
        debug_assert_eq!(bb.length % 8, 0);

        for &padbyte in [0xEC, 0x11].iter().cycle() {
            if bb.length >= datacapacitybits {
                break;
            }
            bb.append_bits(padbyte, 8);
        }
        Ok((bb.length / 8, ecl, version))
    }

    pub fn encode_codewords<'b>(
        mut datacodewordsandoutbuffer: &'a mut [u8],
        datacodewordslen: usize,
        mut tempbuffer: &'b mut [u8],
        ecl: QrCodeEcc,
        version: Version,
        mut msk: Option<Mask>,
    ) -> QrCode<'a> {
        datacodewordsandoutbuffer = &mut datacodewordsandoutbuffer[..version.buffer_len()];
        tempbuffer = &mut tempbuffer[..version.buffer_len()];

        let rawcodewords: usize = QrCode::get_num_raw_data_modules(version) / 8;
        assert!(datacodewordslen <= rawcodewords);
        let (data, temp) = datacodewordsandoutbuffer.split_at_mut(datacodewordslen);
        let allcodewords = Self::add_ecc_and_interleave(data, version, ecl, temp, tempbuffer);

        let mut result: QrCode =
            QrCode::<'a>::function_modules_marked(datacodewordsandoutbuffer, version);
        result.draw_codewords(allcodewords);
        result.draw_light_function_modules();
        let funcmods: QrCode = QrCode::<'b>::function_modules_marked(tempbuffer, version);

        if msk.is_none() {
            let mut minpenalty = core::i32::MAX;
            for i in 0u8..8 {
                let i = Mask::new(i);
                result.apply_mask(&funcmods, i);
                result.draw_format_bits(ecl, i);
                let penalty: i32 = result.get_penalty_score();
                if penalty < minpenalty {
                    msk = Some(i);
                    minpenalty = penalty;
                }
                result.apply_mask(&funcmods, i);
            }
        }
        let msk: Mask = msk.unwrap();
        result.apply_mask(&funcmods, msk);
        result.draw_format_bits(ecl, msk);
        result
    }

    pub fn version(&self) -> Version {
        Version::new((*self.size - 17) / 4)
    }

    pub fn size(&self) -> i32 {
        i32::from(*self.size)
    }

    pub fn error_correction_level(&self) -> QrCodeEcc {
        let index = usize::from(self.get_module_bounded(0, 8)) << 1
            | usize::from(self.get_module_bounded(1, 8));
        use QrCodeEcc::*;
        [Medium, Low, High, Quartile][index]
    }

    pub fn mask(&self) -> Mask {
        Mask::new(
            u8::from(self.get_module_bounded(2, 8)) << 2
                | u8::from(self.get_module_bounded(3, 8)) << 1
                | u8::from(self.get_module_bounded(4, 8)),
        )
    }

    pub fn get_module(&self, x: i32, y: i32) -> bool {
        let range = 0..self.size();
        range.contains(&x) && range.contains(&y) && self.get_module_bounded(x as u8, y as u8)
    }

    fn get_module_bounded(&self, x: u8, y: u8) -> bool {
        let range = 0..*self.size;
        assert!(range.contains(&x) && range.contains(&y));
        let index = usize::from(y) * usize::from(*self.size) + usize::from(x);
        let byteindex: usize = index >> 3;
        let bitindex: usize = index & 7;
        get_bit(self.modules[byteindex].into(), bitindex as u8)
    }

    fn set_module_unbounded(&mut self, x: i32, y: i32, isdark: bool) {
        let range = 0..self.size();
        if range.contains(&x) && range.contains(&y) {
            self.set_module_bounded(x as u8, y as u8, isdark);
        }
    }

    fn set_module_bounded(&mut self, x: u8, y: u8, isdark: bool) {
        let range = 0..*self.size;
        assert!(range.contains(&x) && range.contains(&y));
        let index = usize::from(y) * usize::from(*self.size) + usize::from(x);
        let byteindex: usize = index >> 3;
        let bitindex: usize = index & 7;
        if isdark {
            self.modules[byteindex] |= 1u8 << bitindex;
        } else {
            self.modules[byteindex] &= !(1u8 << bitindex);
        }
    }

    fn add_ecc_and_interleave<'b>(
        data: &[u8],
        ver: Version,
        ecl: QrCodeEcc,
        temp: &mut [u8],
        resultbuf: &'b mut [u8],
    ) -> &'b [u8] {
        assert_eq!(data.len(), QrCode::get_num_data_codewords(ver, ecl));

        let numblocks: usize = QrCode::table_get(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl);
        let blockecclen: usize = QrCode::table_get(&ECC_CODEWORDS_PER_BLOCK, ver, ecl);
        let rawcodewords: usize = QrCode::get_num_raw_data_modules(ver) / 8;
        let numshortblocks: usize = numblocks - rawcodewords % numblocks;
        let shortblockdatalen: usize = rawcodewords / numblocks - blockecclen;
        let result = &mut resultbuf[..rawcodewords];

        let rs = ReedSolomonGenerator::new(blockecclen);
        let mut dat: &[u8] = data;
        let ecc: &mut [u8] = &mut temp[..blockecclen];
        for i in 0..numblocks {
            let datlen: usize = shortblockdatalen + usize::from(i >= numshortblocks);
            rs.compute_remainder(&dat[..datlen], ecc);
            let mut k: usize = i;
            for j in 0..datlen {
                if j == shortblockdatalen {
                    k -= numshortblocks;
                }
                result[k] = dat[j];
                k += numblocks;
            }
            let mut k: usize = data.len() + i;
            for j in 0..blockecclen {
                result[k] = ecc[j];
                k += numblocks;
            }
            dat = &dat[datlen..];
        }
        debug_assert_eq!(dat.len(), 0);
        result
    }

    fn function_modules_marked(outbuffer: &'a mut [u8], ver: Version) -> Self {
        assert_eq!(outbuffer.len(), ver.buffer_len());
        let parts: (&mut u8, &mut [u8]) = outbuffer.split_first_mut().unwrap();
        let mut result = Self {
            size: parts.0,
            modules: parts.1,
        };
        let size: u8 = ver.value() * 4 + 17;
        *result.size = size;
        result.modules.fill(0);

        result.fill_rectangle(6, 0, 1, size);
        result.fill_rectangle(0, 6, size, 1);

        result.fill_rectangle(0, 0, 9, 9);
        result.fill_rectangle(size - 8, 0, 8, 9);
        result.fill_rectangle(0, size - 8, 9, 8);

        let mut alignpatposbuf = [0u8; 7];
        let alignpatpos: &[u8] = result.get_alignment_pattern_positions(&mut alignpatposbuf);
        for (i, pos0) in alignpatpos.iter().enumerate() {
            for (j, pos1) in alignpatpos.iter().enumerate() {
                if !((i == 0 && j == 0)
                    || (i == 0 && j == alignpatpos.len() - 1)
                    || (i == alignpatpos.len() - 1 && j == 0))
                {
                    result.fill_rectangle(pos0 - 2, pos1 - 2, 5, 5);
                }
            }
        }

        if ver.value() >= 7 {
            result.fill_rectangle(size - 11, 0, 3, 6);
            result.fill_rectangle(0, size - 11, 6, 3);
        }

        result
    }

    fn draw_light_function_modules(&mut self) {
        let size: u8 = *self.size;
        for i in (7..size - 7).step_by(2) {
            self.set_module_bounded(6, i, false);
            self.set_module_bounded(i, 6, false);
        }

        for dy in -4i32..=4 {
            for dx in -4i32..=4 {
                let dist: i32 = dx.abs().max(dy.abs());
                if dist == 2 || dist == 4 {
                    self.set_module_unbounded(3 + dx, 3 + dy, false);
                    self.set_module_unbounded(i32::from(size) - 4 + dx, 3 + dy, false);
                    self.set_module_unbounded(3 + dx, i32::from(size) - 4 + dy, false);
                }
            }
        }

        let mut alignpatposbuf = [0u8; 7];
        let alignpatpos: &[u8] = self.get_alignment_pattern_positions(&mut alignpatposbuf);
        for (i, &pos0) in alignpatpos.iter().enumerate() {
            for (j, &pos1) in alignpatpos.iter().enumerate() {
                if (i == 0 && j == 0)
                    || (i == 0 && j == alignpatpos.len() - 1)
                    || (i == alignpatpos.len() - 1 && j == 0)
                {
                    continue;
                }
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        self.set_module_bounded(
                            (i32::from(pos0) + dx) as u8,
                            (i32::from(pos1) + dy) as u8,
                            dx == 0 && dy == 0,
                        );
                    }
                }
            }
        }

        let ver = u32::from(self.version().value());
        if ver >= 7 {
            let bits: u32 = {
                let mut rem: u32 = ver;
                for _ in 0..12 {
                    rem = (rem << 1) ^ ((rem >> 11) * 0x1F25);
                }
                ver << 12 | rem
            };
            debug_assert_eq!(bits >> 18, 0);

            for i in 0u8..18 {
                let bit: bool = get_bit(bits, i);
                let a: u8 = size - 11 + i % 3;
                let b: u8 = i / 3;
                self.set_module_bounded(a, b, bit);
                self.set_module_bounded(b, a, bit);
            }
        }
    }

    fn draw_format_bits(&mut self, ecl: QrCodeEcc, mask: Mask) {
        let bits: u32 = {
            let data = u32::from(ecl.format_bits() << 3 | mask.value());
            let mut rem: u32 = data;
            for _ in 0..10 {
                rem = (rem << 1) ^ ((rem >> 9) * 0x537);
            }
            (data << 10 | rem) ^ 0x5412
        };
        debug_assert_eq!(bits >> 15, 0);

        for i in 0..6 {
            self.set_module_bounded(8, i, get_bit(bits, i));
        }
        self.set_module_bounded(8, 7, get_bit(bits, 6));
        self.set_module_bounded(8, 8, get_bit(bits, 7));
        self.set_module_bounded(7, 8, get_bit(bits, 8));
        for i in 9..15 {
            self.set_module_bounded(14 - i, 8, get_bit(bits, i));
        }

        let size: u8 = *self.size;
        for i in 0..8 {
            self.set_module_bounded(size - 1 - i, 8, get_bit(bits, i));
        }
        for i in 8..15 {
            self.set_module_bounded(8, size - 15 + i, get_bit(bits, i));
        }
        self.set_module_bounded(8, size - 8, true);
    }

    fn fill_rectangle(&mut self, left: u8, top: u8, width: u8, height: u8) {
        for dy in 0..height {
            for dx in 0..width {
                self.set_module_bounded(left + dx, top + dy, true);
            }
        }
    }

    fn draw_codewords(&mut self, data: &[u8]) {
        assert_eq!(
            data.len(),
            QrCode::get_num_raw_data_modules(self.version()) / 8,
            "Illegal argument"
        );

        let size: i32 = self.size();
        let mut i: usize = 0;
        let mut right: i32 = size - 1;
        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            for vert in 0..size {
                for j in 0..2 {
                    let x = (right - j) as u8;
                    let upward: bool = (right + 1) & 2 == 0;
                    let y = (if upward { size - 1 - vert } else { vert }) as u8;
                    if !self.get_module_bounded(x, y) && i < data.len() * 8 {
                        self.set_module_bounded(
                            x,
                            y,
                            get_bit(data[i >> 3].into(), 7 - ((i as u8) & 7)),
                        );
                        i += 1;
                    }
                }
            }
            right -= 2;
        }
        debug_assert_eq!(i, data.len() * 8);
    }

    fn apply_mask(&mut self, functionmodules: &QrCode, mask: Mask) {
        for y in 0..*self.size {
            for x in 0..*self.size {
                if functionmodules.get_module_bounded(x, y) {
                    continue;
                }
                let invert: bool = {
                    let x = i32::from(x);
                    let y = i32::from(y);
                    match mask.value() {
                        0 => (x + y) % 2 == 0,
                        1 => y % 2 == 0,
                        2 => x % 3 == 0,
                        3 => (x + y) % 3 == 0,
                        4 => (x / 3 + y / 2) % 2 == 0,
                        5 => x * y % 2 + x * y % 3 == 0,
                        6 => (x * y % 2 + x * y % 3) % 2 == 0,
                        7 => ((x + y) % 2 + x * y % 3) % 2 == 0,
                        _ => unreachable!(),
                    }
                };
                self.set_module_bounded(x, y, self.get_module_bounded(x, y) ^ invert);
            }
        }
    }

    fn get_penalty_score(&self) -> i32 {
        let mut result: i32 = 0;
        let size: u8 = *self.size;

        for y in 0..size {
            let mut runcolor = false;
            let mut runx: i32 = 0;
            let mut runhistory = FinderPenalty::new(size);
            for x in 0..size {
                if self.get_module_bounded(x, y) == runcolor {
                    runx += 1;
                    if runx == 5 {
                        result += PENALTY_N1;
                    } else if runx > 5 {
                        result += 1;
                    }
                } else {
                    runhistory.add_history(runx);
                    if !runcolor {
                        result += runhistory.count_patterns() * PENALTY_N3;
                    }
                    runcolor = self.get_module_bounded(x, y);
                    runx = 1;
                }
            }
            result += runhistory.terminate_and_count(runcolor, runx) * PENALTY_N3;
        }

        for x in 0..size {
            let mut runcolor = false;
            let mut runy: i32 = 0;
            let mut runhistory = FinderPenalty::new(size);
            for y in 0..size {
                if self.get_module_bounded(x, y) == runcolor {
                    runy += 1;
                    if runy == 5 {
                        result += PENALTY_N1;
                    } else if runy > 5 {
                        result += 1;
                    }
                } else {
                    runhistory.add_history(runy);
                    if !runcolor {
                        result += runhistory.count_patterns() * PENALTY_N3;
                    }
                    runcolor = self.get_module_bounded(x, y);
                    runy = 1;
                }
            }
            result += runhistory.terminate_and_count(runcolor, runy) * PENALTY_N3;
        }

        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let color: bool = self.get_module_bounded(x, y);
                if color == self.get_module_bounded(x + 1, y)
                    && color == self.get_module_bounded(x, y + 1)
                    && color == self.get_module_bounded(x + 1, y + 1)
                {
                    result += PENALTY_N2;
                }
            }
        }

        let dark = self.modules.iter().map(|x| x.count_ones()).sum::<u32>() as i32;
        let total = i32::from(size) * i32::from(size);
        let k: i32 = ((dark * 20 - total * 10).abs() + total - 1) / total - 1;
        debug_assert!((0..=9).contains(&k));
        result += k * PENALTY_N4;
        debug_assert!((0..=2_568_888).contains(&result));
        result
    }

    fn get_alignment_pattern_positions<'b>(&self, resultbuf: &'b mut [u8; 7]) -> &'b [u8] {
        let ver: u8 = self.version().value();
        if ver == 1 {
            &resultbuf[..0]
        } else {
            let numalign: u8 = ver / 7 + 2;
            let step = u8::try_from(
                (i32::from(ver) * 8 + i32::from(numalign) * 3 + 5) / (i32::from(numalign) * 4 - 4)
                    * 2,
            )
            .unwrap();
            let result = &mut resultbuf[..usize::from(numalign)];
            for i in 0..numalign - 1 {
                result[usize::from(i)] = *self.size - 7 - i * step;
            }
            *result.last_mut().unwrap() = 6;
            result.reverse();
            result
        }
    }

    fn get_num_raw_data_modules(ver: Version) -> usize {
        let ver = usize::from(ver.value());
        let mut result: usize = (16 * ver + 128) * ver + 64;
        if ver >= 2 {
            let numalign: usize = ver / 7 + 2;
            result -= (25 * numalign - 10) * numalign - 55;
            if ver >= 7 {
                result -= 36;
            }
        }
        debug_assert!((208..=29_648).contains(&result));
        result
    }

    fn get_num_data_codewords(ver: Version, ecl: QrCodeEcc) -> usize {
        QrCode::get_num_raw_data_modules(ver) / 8
            - QrCode::table_get(&ECC_CODEWORDS_PER_BLOCK, ver, ecl)
                * QrCode::table_get(&NUM_ERROR_CORRECTION_BLOCKS, ver, ecl)
    }

    fn table_get(table: &'static [[i8; 41]; 4], ver: Version, ecl: QrCodeEcc) -> usize {
        table[ecl.ordinal()][usize::from(ver.value())] as usize
    }
}

impl PartialEq for QrCode<'_> {
    fn eq(&self, other: &QrCode<'_>) -> bool {
        *self.size == *other.size && *self.modules == *other.modules
    }
}

impl Eq for QrCode<'_> {}

struct ReedSolomonGenerator {
    divisor: [u8; 30],
    degree: usize,
}

impl ReedSolomonGenerator {
    fn new(degree: usize) -> Self {
        let mut result = Self {
            divisor: [0u8; 30],
            degree,
        };
        assert!(
            (1..=result.divisor.len()).contains(&degree),
            "Degree out of range"
        );
        let divisor: &mut [u8] = &mut result.divisor[..degree];
        divisor[degree - 1] = 1;

        let mut root: u8 = 1;
        for _ in 0..degree {
            for j in 0..degree {
                divisor[j] = Self::multiply(divisor[j], root);
                if j + 1 < divisor.len() {
                    divisor[j] ^= divisor[j + 1];
                }
            }
            root = Self::multiply(root, 0x02);
        }
        result
    }

    fn compute_remainder(&self, data: &[u8], result: &mut [u8]) {
        assert_eq!(result.len(), self.degree);
        result.fill(0);
        for b in data {
            let factor: u8 = b ^ result[0];
            result.copy_within(1.., 0);
            result[result.len() - 1] = 0;
            for (x, &y) in result.iter_mut().zip(self.divisor.iter()) {
                *x ^= Self::multiply(y, factor);
            }
        }
    }

    fn multiply(x: u8, y: u8) -> u8 {
        let mut z: u8 = 0;
        for i in (0..8).rev() {
            z = (z << 1) ^ ((z >> 7) * 0x1D);
            z ^= ((y >> i) & 1) * x;
        }
        z
    }
}

struct FinderPenalty {
    qr_size: i32,
    run_history: [i32; 7],
}

impl FinderPenalty {
    pub fn new(size: u8) -> Self {
        Self {
            qr_size: i32::from(size),
            run_history: [0; 7],
        }
    }

    pub fn add_history(&mut self, mut currentrunlength: i32) {
        if self.run_history[0] == 0 {
            currentrunlength += self.qr_size;
        }
        let len: usize = self.run_history.len();
        self.run_history.copy_within(0..len - 1, 1);
        self.run_history[0] = currentrunlength;
    }

    pub fn count_patterns(&self) -> i32 {
        let rh = &self.run_history;
        let n = rh[1];
        debug_assert!(n <= self.qr_size * 3);
        let core = n > 0 && rh[2] == n && rh[3] == n * 3 && rh[4] == n && rh[5] == n;
        #[allow(unused_parens)]
        (i32::from(core && rh[0] >= n * 4 && rh[6] >= n)
            + i32::from(core && rh[6] >= n * 4 && rh[0] >= n))
    }

    pub fn terminate_and_count(mut self, currentruncolor: bool, mut currentrunlength: i32) -> i32 {
        if currentruncolor {
            self.add_history(currentrunlength);
            currentrunlength = 0;
        }
        currentrunlength += self.qr_size;
        self.add_history(currentrunlength);
        self.count_patterns()
    }
}

const PENALTY_N1: i32 = 3;
const PENALTY_N2: i32 = 3;
const PENALTY_N3: i32 = 40;
const PENALTY_N4: i32 = 10;

static ECC_CODEWORDS_PER_BLOCK: [[i8; 41]; 4] = [
    [
        -1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        -1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
    [
        -1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30,
        30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        -1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
];

static NUM_ERROR_CORRECTION_BLOCKS: [[i8; 41]; 4] = [
    [
        -1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12,
        13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        -1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
    [
        -1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27,
        29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ],
    [
        -1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32,
        35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ],
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum QrCodeEcc {
    Low,
    Medium,
    Quartile,
    High,
}

impl QrCodeEcc {
    fn ordinal(self) -> usize {
        use QrCodeEcc::*;
        match self {
            Low => 0,
            Medium => 1,
            Quartile => 2,
            High => 3,
        }
    }

    fn format_bits(self) -> u8 {
        use QrCodeEcc::*;
        match self {
            Low => 1,
            Medium => 0,
            Quartile => 3,
            High => 2,
        }
    }
}

pub struct QrSegment<'a> {
    mode: QrSegmentMode,
    numchars: usize,
    data: &'a [u8],
    bitlength: usize,
}

impl<'a> QrSegment<'a> {
    pub fn make_bytes(data: &'a [u8]) -> Self {
        QrSegment::new(
            QrSegmentMode::Byte,
            data.len(),
            data,
            data.len().checked_mul(8).unwrap(),
        )
    }

    pub fn make_numeric(text: &str, buf: &'a mut [u8]) -> Self {
        assert!(
            text.bytes().all(|b| (b'0'..=b'9').contains(&b)),
            "String contains non-numeric characters"
        );
        let mut bb = BitBuffer::new(buf);
        for chunk in text.as_bytes().chunks(3) {
            let data: u32 = chunk
                .iter()
                .fold(0u32, |acc, &b| acc * 10 + u32::from(b - b'0'));
            bb.append_bits(data, (chunk.len() as u8) * 3 + 1);
        }
        QrSegment::new(QrSegmentMode::Numeric, text.len(), bb.data, bb.length)
    }

    pub fn make_alphanumeric(text: &str, buf: &'a mut [u8]) -> Self {
        let mut bb = BitBuffer::new(buf);
        for chunk in text.as_bytes().chunks(2) {
            let data: u32 = chunk.iter().fold(0u32, |acc, &b| {
                acc * 45
                    + u32::try_from(
                        ALPHANUMERIC_CHARSET
                            .find(char::from(b))
                            .expect("String contains unencodable characters in alphanumeric mode"),
                    )
                    .unwrap()
            });
            bb.append_bits(data, (chunk.len() as u8) * 5 + 1);
        }
        QrSegment::new(QrSegmentMode::Alphanumeric, text.len(), bb.data, bb.length)
    }

    pub fn make_eci(assignval: u32, buf: &'a mut [u8]) -> Self {
        let mut bb = BitBuffer::new(buf);
        if assignval < (1 << 7) {
            bb.append_bits(assignval, 8);
        } else if assignval < (1 << 14) {
            bb.append_bits(0b10, 2);
            bb.append_bits(assignval, 14);
        } else if assignval < 1_000_000 {
            bb.append_bits(0b110, 3);
            bb.append_bits(assignval, 21);
        } else {
            panic!("ECI assignment value out of range");
        }
        QrSegment::new(QrSegmentMode::Eci, 0, bb.data, bb.length)
    }

    pub fn new(mode: QrSegmentMode, numchars: usize, data: &'a [u8], bitlength: usize) -> Self {
        assert!(bitlength == 0 || (bitlength - 1) / 8 < data.len());
        Self {
            mode,
            numchars,
            data,
            bitlength,
        }
    }

    pub fn mode(&self) -> QrSegmentMode {
        self.mode
    }

    pub fn num_chars(&self) -> usize {
        self.numchars
    }

    pub fn calc_buffer_size(mode: QrSegmentMode, numchars: usize) -> Option<usize> {
        let temp = Self::calc_bit_length(mode, numchars)?;
        Some(temp / 8 + usize::from(temp % 8 != 0))
    }

    fn calc_bit_length(mode: QrSegmentMode, numchars: usize) -> Option<usize> {
        let mul_frac_ceil = |numer: usize, denom: usize| {
            Some(numchars)
                .and_then(|x| x.checked_mul(numer))
                .and_then(|x| x.checked_add(denom - 1))
                .map(|x| x / denom)
        };

        use QrSegmentMode::*;
        match mode {
            Numeric => mul_frac_ceil(10, 3),
            Alphanumeric => mul_frac_ceil(11, 2),
            Byte => mul_frac_ceil(8, 1),
            Kanji => mul_frac_ceil(13, 1),
            Eci => {
                assert_eq!(numchars, 0);
                Some(3 * 8)
            }
        }
    }

    fn get_total_bits(segs: &[Self], version: Version) -> Option<usize> {
        let mut result: usize = 0;
        for seg in segs {
            let ccbits: u8 = seg.mode.num_char_count_bits(version);
            if let Some(limit) = 1usize.checked_shl(ccbits.into()) {
                if seg.numchars >= limit {
                    return None;
                }
            }
            result = result.checked_add(4 + usize::from(ccbits))?;
            result = result.checked_add(seg.bitlength)?;
        }
        Some(result)
    }

    pub fn is_numeric(text: &str) -> bool {
        text.chars().all(|c| ('0'..='9').contains(&c))
    }

    pub fn is_alphanumeric(text: &str) -> bool {
        text.chars().all(|c| ALPHANUMERIC_CHARSET.contains(c))
    }
}

static ALPHANUMERIC_CHARSET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QrSegmentMode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
    Eci,
}

impl QrSegmentMode {
    fn mode_bits(self) -> u32 {
        use QrSegmentMode::*;
        match self {
            Numeric => 0x1,
            Alphanumeric => 0x2,
            Byte => 0x4,
            Kanji => 0x8,
            Eci => 0x7,
        }
    }

    fn num_char_count_bits(self, ver: Version) -> u8 {
        use QrSegmentMode::*;
        (match self {
            Numeric => [10, 12, 14],
            Alphanumeric => [9, 11, 13],
            Byte => [8, 16, 16],
            Kanji => [8, 10, 12],
            Eci => [0, 0, 0],
        })[usize::from((ver.value() + 7) / 17)]
    }
}

pub struct BitBuffer<'a> {
    data: &'a mut [u8],
    length: usize,
}

impl<'a> BitBuffer<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            data: buffer,
            length: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn append_bits(&mut self, val: u32, len: u8) {
        assert!(len <= 31 && val >> len == 0);
        assert!(usize::from(len) <= usize::MAX - self.length);
        for i in (0..len).rev() {
            let index: usize = self.length >> 3;
            let shift: u8 = 7 - ((self.length as u8) & 7);
            let bit: u8 = ((val >> i) as u8) & 1;
            if shift == 7 {
                self.data[index] = bit << shift;
            } else {
                self.data[index] |= bit << shift;
            }
            self.length += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub enum DataTooLong {
    SegmentTooLong,
    DataOverCapacity(usize, usize),
}

impl core::fmt::Display for DataTooLong {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Self::SegmentTooLong => write!(f, "Segment too long"),
            Self::DataOverCapacity(datalen, maxcapacity) => {
                write!(
                    f,
                    "Data length = {} bits, Max capacity = {} bits",
                    datalen, maxcapacity
                )
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Version(u8);

impl Version {
    pub const MIN: Version = Version(1);
    pub const MAX: Version = Version(40);

    pub const fn new(ver: u8) -> Self {
        assert!(
            Version::MIN.value() <= ver && ver <= Version::MAX.value(),
            "Version number out of range"
        );
        Self(ver)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn buffer_len(self) -> usize {
        let sidelen = (self.0 as usize) * 4 + 17;
        (sidelen * sidelen + 7) / 8 + 1
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Mask(u8);

impl Mask {
    pub const fn new(mask: u8) -> Self {
        assert!(mask <= 7, "Mask value out of range");
        Self(mask)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

fn get_bit(x: u32, i: u8) -> bool {
    (x >> i) & 1 != 0
}
