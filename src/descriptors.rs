use core::{
    cmp,
    convert::{TryFrom, TryInto},
    fmt,
};

use encoding::{all::ISO_8859_1, EncoderTrap, Encoding};
use num_traits::{Bounded, CheckedShl, Euclid, FromPrimitive, Num, WrappingSub};
use typed_builder::TypedBuilder;

use crate::{
    EdidTypeConversionError, IntoBytes, EDID_DESCRIPTORS_NUM, EDID_DESCRIPTOR_LEN,
    EDID_DESCRIPTOR_PAYLOAD_LEN,
};

pub(crate) fn round_up<T>(number: T, multiple: T) -> T
where
    T: Num + Euclid + FromPrimitive,
{
    let rem = number.rem_euclid(&multiple);

    if rem.is_zero() {
        return number;
    }

    let div = number.div_euclid(&multiple) + T::one();
    div * multiple
}

fn compute_max_value<T>(num_bits: u32) -> T
where
    T: Num + Bounded + CheckedShl + WrappingSub,
{
    let type_num_bits = u32::try_from(std::mem::size_of::<T>() * 8).unwrap();

    match num_bits.cmp(&type_num_bits) {
        cmp::Ordering::Less => {
            let shl = T::checked_shl(&T::one(), num_bits).unwrap();
            T::wrapping_sub(&shl, &T::one())
        }
        cmp::Ordering::Equal => T::max_value(),
        cmp::Ordering::Greater => unreachable!(),
    }
}

#[cfg(test)]
mod test_max_size_bits {
    use super::compute_max_value;

    #[test]
    fn test() {
        assert_eq!(compute_max_value::<u8>(4), 0xf);
        assert_eq!(compute_max_value::<u32>(4), 0xf);
        assert_eq!(compute_max_value::<u8>(8), 0xff);
        assert_eq!(compute_max_value::<u32>(32), 0xffff_ffff);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDescriptorCustomTag(u8);

impl TryFrom<u8> for EdidDescriptorCustomTag {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 0x0f {
            Err(EdidTypeConversionError::Range(value, Some(0), Some(0x10)))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdidDescriptorCustomPayload(Vec<u8>);

impl TryFrom<Vec<u8>> for EdidDescriptorCustomPayload {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() > EDID_DESCRIPTOR_PAYLOAD_LEN {
            Err(EdidTypeConversionError::Value(String::from(
                "Custom Descriptor Payload must be at most 13 bytes long.",
            )))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug)]
pub struct EdidDescriptorCustom {
    tag: EdidDescriptorCustomTag,
    payload: EdidDescriptorCustomPayload,
}

impl IntoBytes for EdidDescriptorCustom {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

        let tag = self.tag.0;
        bytes.extend_from_slice(&[0, 0, 0, tag, 0]);
        bytes.extend_from_slice(&self.payload.0);
        bytes.resize(EDID_DESCRIPTOR_LEN, 0);

        bytes
    }
}

impl TryFrom<(u8, Vec<u8>)> for EdidDescriptorCustom {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: (u8, Vec<u8>)) -> Result<Self, Self::Error> {
        let tag = value.0.try_into()?;
        let payload = value.1.try_into()?;

        Ok(Self { tag, payload })
    }
}

#[derive(Clone, Debug)]
pub struct EdidDescriptorString(String);

impl EdidDescriptorString {
    /// Some EDIDs in the test suite use non-ASCII characters, going against the spec. We want to
    /// prevent that from happening for new EDIDs, but we still need to allow to build our string
    /// for our tests.
    #[must_use]
    #[doc(hidden)]
    pub fn from_str_encoding_unchecked(value: &str) -> Self {
        let len = value.chars().count();
        assert!(len <= EDID_DESCRIPTOR_PAYLOAD_LEN, "String is too long");

        Self(String::from(value))
    }
}

impl TryFrom<String> for EdidDescriptorString {
    type Error = EdidTypeConversionError<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
            return Err(EdidTypeConversionError::Value(String::from(
                "String must be ASCII.",
            )));
        }

        // Strictly speaking, a String length in bytes is different than its number of characters.
        // However, because we checked that we only have ASCII characters, we have that 1-byte ->
        // 1-char guarantee.
        if value.len() > EDID_DESCRIPTOR_PAYLOAD_LEN {
            return Err(EdidTypeConversionError::Value(String::from(
                "String is too long.",
            )));
        }

        Ok(Self(value))
    }
}

impl TryFrom<&str> for EdidDescriptorString {
    type Error = EdidTypeConversionError<String>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        String::from(value).try_into()
    }
}

impl IntoBytes for EdidDescriptorString {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_PAYLOAD_LEN);

        // A Rust String is in UTF-8, an EDID String is supposed to be ASCII-only. Some EDIDs
        // deviate from that so we still need to output an ASCII-ish bytes array, but without the
        // Unicode leading bytes. ISO-8859-1 seems like a good enough guess at the moment.
        let iso_bytes = ISO_8859_1
            .encode(&self.0, EncoderTrap::Strict)
            .expect("String Encoding failed.");
        bytes.extend_from_slice(&iso_bytes);

        if bytes.len() < EDID_DESCRIPTOR_PAYLOAD_LEN {
            bytes.push(0x0a);
        }

        bytes.resize(EDID_DESCRIPTOR_PAYLOAD_LEN, 0x20);

        assert!(
            bytes.len() == EDID_DESCRIPTOR_PAYLOAD_LEN,
            "Serialized String Representation is too large ({} vs expected {})",
            bytes.len(),
            EDID_DESCRIPTOR_PAYLOAD_LEN
        );
        bytes
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDetailedTimingPixelClock(u32);

impl EdidDetailedTimingPixelClock {
    fn into_raw(self) -> u16 {
        u16::try_from(self.0 / 10).expect("Detailed Timing Pixel clock would overflow our type")
    }
}

impl TryFrom<u32> for EdidDetailedTimingPixelClock {
    type Error = EdidTypeConversionError<u32>;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if !(10..=655_350).contains(&value) {
            return Err(EdidTypeConversionError::Range(
                value,
                Some(10),
                Some(655350),
            ));
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod test_descriptor_detailed_timing_pixel_clock {
    use super::EdidDetailedTimingPixelClock;

    #[test]
    fn test_binary_spec() {
        // Taken from EDID 1.4 Specification, Section 3.10.2
        let clk = EdidDetailedTimingPixelClock::try_from(135_000).unwrap();
        assert_eq!(clk.into_raw().to_le_bytes(), [0xbc, 0x34]);
    }

    #[test]
    fn test_range() {
        assert!(EdidDetailedTimingPixelClock::try_from(0).is_err());
        assert!(EdidDetailedTimingPixelClock::try_from(1).is_err());
        assert!(EdidDetailedTimingPixelClock::try_from(10).is_ok());
        assert!(EdidDetailedTimingPixelClock::try_from(655_350).is_ok());
        assert!(EdidDetailedTimingPixelClock::try_from(655_351).is_err());
        assert!(EdidDetailedTimingPixelClock::try_from(u32::MAX).is_err());

    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidDetailedTimingAnalogSync {
    BipolarComposite(bool, bool),
    Composite(bool, bool),
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidDetailedTimingDigitalCompositeSync {
    #[builder(default)]
    serrations: bool,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidDetailedTimingDigitalSeparateSync {
    #[builder(default)]
    vsync_positive: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum EdidDetailedTimingDigitalSyncKind {
    Composite(EdidDetailedTimingDigitalCompositeSync),
    Separate(EdidDetailedTimingDigitalSeparateSync),
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidDetailedTimingDigitalSync {
    kind: EdidDetailedTimingDigitalSyncKind,

    #[builder(default)]
    hsync_positive: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum EdidDetailedTimingSync {
    Analog(EdidDetailedTimingAnalogSync),
    Digital(EdidDetailedTimingDigitalSync),
}

#[derive(Clone, Copy, Debug)]
pub enum EdidDetailedTimingStereo {
    None,
    FieldSequentialRightOnSync,
    FieldSequentialLeftOnSync,
    TwoWayInterleavedRightOnEven,
    TwoWayInterleavedLeftOnEven,
    FourWayInterleaved,
    SideBySideInterleaved,
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDescriptorTiming<const N: u32, T: fmt::Display>(T);

impl<const N: u32, T> EdidDescriptorTiming<N, T>
where
    T: Num + Bounded + CheckedShl + WrappingSub + cmp::PartialOrd + fmt::Display,
{
    fn try_from(value: T) -> Result<Self, EdidTypeConversionError<T>> {
        let max = compute_max_value::<T>(N);

        if value > max {
            return Err(EdidTypeConversionError::Range(value, None, Some(max)));
        }

        Ok(Self(value))
    }

    fn into_raw(self) -> T {
        self.0
    }
}

pub type EdidDescriptor6BitsTiming = EdidDescriptorTiming<6, u8>;

impl TryFrom<u8> for EdidDescriptor6BitsTiming {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        EdidDescriptorTiming::try_from(value)
    }
}

#[cfg(test)]
mod test_edid_detailed_timings_6bits_fields {
    use crate::EdidDescriptor6BitsTiming;

    #[test]
    fn test_range() {
        assert!(EdidDescriptor6BitsTiming::try_from(0).is_ok());
        assert!(EdidDescriptor6BitsTiming::try_from(63).is_ok());
        assert!(EdidDescriptor6BitsTiming::try_from(64).is_err());
        assert!(EdidDescriptor6BitsTiming::try_from(u8::MAX).is_err());
    }
}

pub type EdidDescriptor8BitsTiming = EdidDescriptorTiming<8, u8>;

impl TryFrom<u8> for EdidDescriptor8BitsTiming {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        EdidDescriptorTiming::try_from(value)
    }
}

#[cfg(test)]
mod test_edid_detailed_timings_8bits_fields {
    use crate::EdidDescriptor8BitsTiming;

    #[test]
    fn test_range() {
        assert!(EdidDescriptor8BitsTiming::try_from(0).is_ok());
        assert!(EdidDescriptor8BitsTiming::try_from(255).is_ok());
    }
}

pub type EdidDescriptor10BitsTiming = EdidDescriptorTiming<10, u16>;

impl TryFrom<u16> for EdidDescriptor10BitsTiming {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        EdidDescriptorTiming::try_from(value)
    }
}

#[cfg(test)]
mod test_edid_detailed_timings_10bits_fields {
    use crate::EdidDescriptor10BitsTiming;

    #[test]
    fn test_range() {
        assert!(EdidDescriptor10BitsTiming::try_from(0).is_ok());
        assert!(EdidDescriptor10BitsTiming::try_from(1013).is_ok());
        assert!(EdidDescriptor10BitsTiming::try_from(1024).is_err());
        assert!(EdidDescriptor10BitsTiming::try_from(u16::MAX).is_err());
    }
}

pub type EdidDescriptor12BitsTiming = EdidDescriptorTiming<12, u16>;

impl TryFrom<u16> for EdidDescriptor12BitsTiming {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        EdidDescriptorTiming::try_from(value)
    }
}

#[cfg(test)]
mod test_edid_detailed_timings_12bits_fields {
    use crate::EdidDescriptor12BitsTiming;

    #[test]
    fn test_range() {
        assert!(EdidDescriptor12BitsTiming::try_from(0).is_ok());
        assert!(EdidDescriptor12BitsTiming::try_from(4095).is_ok());
        assert!(EdidDescriptor12BitsTiming::try_from(4096).is_err());
        assert!(EdidDescriptor12BitsTiming::try_from(u16::MAX).is_err());
    }
}

pub type EdidDetailedTimingSizeMm = EdidDescriptor12BitsTiming;

#[cfg(test)]
mod test_edid_detailed_timings_size {
    use crate::EdidDetailedTimingSizeMm;

    #[test]
    fn test_range() {
        assert!(EdidDetailedTimingSizeMm::try_from(0).is_ok());
        assert!(EdidDetailedTimingSizeMm::try_from(4095).is_ok());
        assert!(EdidDetailedTimingSizeMm::try_from(4096).is_err());
        assert!(EdidDetailedTimingSizeMm::try_from(u16::MAX).is_err());
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidDescriptorDetailedTiming {
    pixel_clock: EdidDetailedTimingPixelClock,

    horizontal_addressable: EdidDescriptor12BitsTiming,
    horizontal_blanking: EdidDescriptor12BitsTiming,

    vertical_addressable: EdidDescriptor12BitsTiming,
    vertical_blanking: EdidDescriptor12BitsTiming,

    horizontal_front_porch: EdidDescriptor10BitsTiming,
    horizontal_sync_pulse: EdidDescriptor10BitsTiming,

    vertical_front_porch: EdidDescriptor6BitsTiming,
    vertical_sync_pulse: EdidDescriptor6BitsTiming,

    horizontal_size: EdidDetailedTimingSizeMm,
    vertical_size: EdidDetailedTimingSizeMm,

    horizontal_border: EdidDescriptor8BitsTiming,
    vertical_border: EdidDescriptor8BitsTiming,

    #[builder(default)]
    interlace: bool,

    sync_type: EdidDetailedTimingSync,
    stereo: EdidDetailedTimingStereo,
}

impl IntoBytes for EdidDescriptorDetailedTiming {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

        let freq = self.pixel_clock.into_raw();
        let lo_freq = (freq & 0xff) as u8;
        let hi_freq = ((freq >> 8) & 0xff) as u8;

        data.extend_from_slice(&[lo_freq, hi_freq]);

        let haddr = self.horizontal_addressable.into_raw();
        let haddr_lo = (haddr & 0xff) as u8;
        let haddr_hi = ((haddr >> 8) & 0xf) as u8;

        let hblank = self.horizontal_blanking.into_raw();
        let hblank_lo = (hblank & 0xff) as u8;
        let hblank_hi = ((hblank >> 8) & 0xf) as u8;

        data.extend_from_slice(&[haddr_lo, hblank_lo, (haddr_hi << 4) | hblank_hi]);

        let vaddr = self.vertical_addressable.into_raw();
        let vaddr_lo = (vaddr & 0xff) as u8;
        let vaddr_hi = ((vaddr >> 8) & 0xf) as u8;

        let vblank = self.vertical_blanking.into_raw();
        let vblank_lo = (vblank & 0xff) as u8;
        let vblank_hi = ((vblank >> 8) & 0xf) as u8;

        data.extend_from_slice(&[vaddr_lo, vblank_lo, (vaddr_hi << 4) | vblank_hi]);

        let hfp = self.horizontal_front_porch.into_raw();
        let hfp_lo = (hfp & 0xff) as u8;
        let hfp_hi = ((hfp >> 8) & 0x3) as u8;

        let hsync = self.horizontal_sync_pulse.into_raw();
        let hsync_lo = (hsync & 0xff) as u8;
        let hsync_hi = ((hsync >> 8) & 0x3) as u8;

        let vfp = self.vertical_front_porch.into_raw();
        let vfp_lo = vfp & 0xf;
        let vfp_hi = (vfp >> 4) & 0x3;

        let vsync = self.vertical_sync_pulse.into_raw();
        let vsync_lo = vsync & 0xf;
        let vsync_hi = (vsync >> 4) & 0x3;

        data.extend_from_slice(&[
            hfp_lo,
            hsync_lo,
            (vfp_lo << 4) | vsync_lo,
            (hfp_hi << 6) | (hsync_hi << 4) | (vfp_hi << 2) | vsync_hi,
        ]);

        let hsize = self.horizontal_size.into_raw();
        let hsize_lo = (hsize & 0xff) as u8;
        let hsize_hi = ((hsize >> 8) & 0xf) as u8;

        let vsize = self.vertical_size.into_raw();
        let vsize_lo = (vsize & 0xff) as u8;
        let vsize_hi = ((vsize >> 8) & 0xf) as u8;

        data.extend_from_slice(&[hsize_lo, vsize_lo, (hsize_hi << 4) | vsize_hi]);

        let mut flags: u8 = 0;

        if self.interlace {
            flags |= 1 << 7;
        }

        match self.stereo {
            EdidDetailedTimingStereo::None => flags |= 0,
            EdidDetailedTimingStereo::FieldSequentialRightOnSync => flags |= 0b010_0000,
            EdidDetailedTimingStereo::FieldSequentialLeftOnSync => flags |= 0b100_0000,
            EdidDetailedTimingStereo::TwoWayInterleavedRightOnEven => flags |= 0b010_0001,
            EdidDetailedTimingStereo::TwoWayInterleavedLeftOnEven => flags |= 0b100_0001,
            EdidDetailedTimingStereo::FourWayInterleaved => flags |= 0b110_0000,
            EdidDetailedTimingStereo::SideBySideInterleaved => flags |= 0b110_0001,
        }

        match self.sync_type {
            EdidDetailedTimingSync::Analog(sync) => match sync {
                EdidDetailedTimingAnalogSync::BipolarComposite(serrations, sync_on_rgb) => {
                    flags |= 0b01 << 3;

                    if serrations {
                        flags |= 1 << 2;
                    }

                    if sync_on_rgb {
                        flags |= 1 << 1;
                    }
                }
                EdidDetailedTimingAnalogSync::Composite(serrations, sync_on_rgb) => {
                    flags |= 0b00 << 3;

                    if serrations {
                        flags |= 1 << 2;
                    }

                    if sync_on_rgb {
                        flags |= 1 << 1;
                    }
                }
            },
            EdidDetailedTimingSync::Digital(v) => {
                match v.kind {
                    EdidDetailedTimingDigitalSyncKind::Separate(v) => {
                        flags |= 0b11 << 3;

                        if v.vsync_positive {
                            flags |= 1 << 2;
                        }
                    }
                    EdidDetailedTimingDigitalSyncKind::Composite(v) => {
                        flags |= 0b10 << 3;

                        if v.serrations {
                            flags |= 1 << 2;
                        }
                    }
                };

                if v.hsync_positive {
                    flags |= 1 << 1;
                }
            }
        }

        data.extend_from_slice(&[
            self.horizontal_border.into_raw(),
            self.vertical_border.into_raw(),
            flags,
        ]);

        data
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDisplayRangeHorizontalFreq(u8);

impl TryFrom<u8> for EdidDisplayRangeHorizontalFreq {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(EdidTypeConversionError::Range(value, Some(1), None));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDisplayRangeVerticalFreq(u8);

impl TryFrom<u8> for EdidDisplayRangeVerticalFreq {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(EdidTypeConversionError::Range(value, Some(1), None));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDisplayRangePixelClock(u16);

impl EdidDisplayRangePixelClock {
    fn round(self) -> u16 {
        round_up(self.0, 10)
    }

    fn into_raw(self) -> u8 {
        u8::try_from(self.round() / 10)
            .expect("Display Range Limits Pixel Clock would overflow our type")
    }
}

impl TryFrom<u16> for EdidDisplayRangePixelClock {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(1..=2550).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(2550)));
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod test_descriptor_display_range_pixel_clock {
    use std::convert::TryFrom;

    use super::EdidDisplayRangePixelClock;

    #[test]
    fn test_binary_spec() {
        // EDID 1.3 Specification, Section 3.10.3.
        assert_eq!(
            EdidDisplayRangePixelClock::try_from(130)
                .unwrap()
                .into_raw(),
            0x0d
        );
        assert_eq!(
            EdidDisplayRangePixelClock::try_from(108)
                .unwrap()
                .into_raw(),
            0x0b
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidDisplayRangeVideoTimingsGTFStartFrequency(u16);

impl TryFrom<u16> for EdidDisplayRangeVideoTimingsGTFStartFrequency {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(1..=510).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(510)));
        }

        Ok(Self(value))
    }
}

impl EdidDisplayRangeVideoTimingsGTFStartFrequency {
    fn into_raw(self) -> u8 {
        u8::try_from(self.0 / 2).expect("GTF Start Frequency too large for our type.")
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidDisplayRangeVideoTimingsGTF {
    #[builder(setter(into))]
    horizontal_start_frequency: EdidDisplayRangeVideoTimingsGTFStartFrequency,
    blanking_offset: u8,
    blanking_gradient: u16,
    blanking_scaling_factor: u8,
    blanking_scaling_factor_weighting: u8,
}

#[derive(Clone, Debug)]
pub enum EdidR3DisplayRangeVideoTimingsSupport {
    DefaultGTF,
    SecondaryGTF(EdidDisplayRangeVideoTimingsGTF),
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct EdidR3DisplayRangeLimits {
    min_hfreq: EdidDisplayRangeHorizontalFreq,
    max_hfreq: EdidDisplayRangeHorizontalFreq,
    min_vfreq: EdidDisplayRangeVerticalFreq,
    max_vfreq: EdidDisplayRangeVerticalFreq,
    max_pixelclock: EdidDisplayRangePixelClock,

    timings_support: EdidR3DisplayRangeVideoTimingsSupport,
}

impl IntoBytes for EdidR3DisplayRangeLimits {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_PAYLOAD_LEN);

        bytes.push(self.min_vfreq.0);
        bytes.push(self.max_vfreq.0);
        bytes.push(self.min_hfreq.0);
        bytes.push(self.max_hfreq.0);
        bytes.push(self.max_pixelclock.into_raw());

        match self.timings_support {
            EdidR3DisplayRangeVideoTimingsSupport::DefaultGTF => {
                bytes.extend_from_slice(&[0x00, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20]);
            }
            EdidR3DisplayRangeVideoTimingsSupport::SecondaryGTF(g) => {
                let blank_grad_lo = (g.blanking_gradient & 0xff) as u8;
                let blank_grad_hi = (g.blanking_gradient >> 8) as u8;

                bytes.extend_from_slice(&[
                    0x02,
                    0x00,
                    g.horizontal_start_frequency.into_raw(),
                    g.blanking_offset * 2,
                    blank_grad_lo,
                    blank_grad_hi,
                    g.blanking_scaling_factor,
                    g.blanking_scaling_factor_weighting * 2,
                ]);
            }
        };

        bytes
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidR4DisplayRangeHorizontalFreq(bool, u8);

impl TryFrom<u16> for EdidR4DisplayRangeHorizontalFreq {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(1..=510).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(510)));
        }

        let mut value = value;
        let offset = if value > 255 {
            value -= 255;
            true
        } else {
            false
        };

        Ok(Self(offset, u8::try_from(value)?))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidR4DisplayRangeVerticalFreq(bool, u8);

impl TryFrom<u16> for EdidR4DisplayRangeVerticalFreq {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(1..=510).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(510)));
        }

        let mut value = value;
        let offset = if value > 255 {
            value -= 255;
            true
        } else {
            false
        };

        Ok(Self(offset, u8::try_from(value)?))
    }
}
#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum EdidR4DisplayRangeVideoTimingsAspectRatio {
    Ratio_4_3 = 0,
    Ratio_16_9,
    Ratio_16_10,
    Ratio_5_4,
    Ratio_15_9,
}

pub struct EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff(u8);

impl TryFrom<EdidDisplayRangePixelClock> for EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: EdidDisplayRangePixelClock) -> Result<Self, Self::Error> {
        let rounded = value.round() - value.0;

        if rounded >= 10 {
            return Err(EdidTypeConversionError::Value(String::from(
                "Computed Additional Precision is too large.",
            )));
        }

        Ok(Self(u8::try_from(rounded)?))
    }
}

impl EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff {
    fn into_raw(self) -> u8 {
        self.0 * 4
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(mutators(
    pub fn supported_aspect_ratios(&mut self, ar: Vec<EdidR4DisplayRangeVideoTimingsAspectRatio>) {
        self.supported_aspect_ratios = ar;
    }

    pub fn add_supported_aspect_ratio(&mut self, ar: EdidR4DisplayRangeVideoTimingsAspectRatio) {
        self.supported_aspect_ratios.push(ar);
    }
))]
pub struct EdidR4DisplayRangeVideoTimingsCVTR1 {
    // FIXME: Max 8184 pixels
    maximum_active_pixels_per_line: u16,
    #[builder(via_mutators)]
    supported_aspect_ratios: Vec<EdidR4DisplayRangeVideoTimingsAspectRatio>,
    preferred_aspect_ratio: EdidR4DisplayRangeVideoTimingsAspectRatio,
    standard_cvt_blanking_supported: bool,
    reduced_cvt_blanking_supported: bool,
    horizontal_shrink_supported: bool,
    horizontal_stretch_supported: bool,
    vertical_shrink_supported: bool,
    vertical_stretch_supported: bool,
    #[builder(setter(into))]
    preferred_vertical_refresh_rate: EdidDisplayRangeVerticalFreq,
}

#[derive(Clone, Debug)]
pub enum EdidR4DisplayRangeVideoTimingsCVT {
    R1(EdidR4DisplayRangeVideoTimingsCVTR1),
}

#[derive(Clone, Debug)]
pub enum EdidR4DisplayRangeVideoTimingsSupport {
    DefaultGTF,
    RangeLimitsOnly,
    #[deprecated = "GTF is considered obsolete with EDID 1.4"]
    SecondaryGTF(EdidDisplayRangeVideoTimingsGTF),
    CVTSupported(EdidR4DisplayRangeVideoTimingsCVT),
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct EdidR4DisplayRangeLimits {
    #[builder(setter(into))]
    min_hfreq: EdidR4DisplayRangeHorizontalFreq,

    #[builder(setter(into))]
    max_hfreq: EdidR4DisplayRangeHorizontalFreq,

    #[builder(setter(into))]
    min_vfreq: EdidR4DisplayRangeVerticalFreq,

    #[builder(setter(into))]
    max_vfreq: EdidR4DisplayRangeVerticalFreq,

    #[builder(setter(into))]
    max_pixelclock: EdidDisplayRangePixelClock,

    timings_support: EdidR4DisplayRangeVideoTimingsSupport,
}

impl IntoBytes for EdidR4DisplayRangeLimits {
    fn into_bytes(self) -> Vec<u8> {
        // The Display Range Limits block has a header a byte shorter than other descriptors.
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_PAYLOAD_LEN + 1);

        let mut flags_byte = 0;
        if self.max_vfreq.0 {
            flags_byte |= 1 << 1;

            if self.min_vfreq.0 {
                flags_byte |= 1 << 0;
            }
        }

        if self.max_hfreq.0 {
            flags_byte |= 1 << 3;

            if self.min_hfreq.0 {
                flags_byte |= 1 << 2;
            }
        }

        bytes.push(flags_byte);
        bytes.push(self.min_vfreq.1);
        bytes.push(self.max_vfreq.1);
        bytes.push(self.min_hfreq.1);
        bytes.push(self.max_hfreq.1);
        bytes.push(self.max_pixelclock.into_raw());

        match self.timings_support {
            EdidR4DisplayRangeVideoTimingsSupport::DefaultGTF => {
                bytes.extend_from_slice(&[0x00, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20]);
            }
            EdidR4DisplayRangeVideoTimingsSupport::RangeLimitsOnly => {
                bytes.extend_from_slice(&[0x01, 0x0a, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20]);
            }
            #[allow(deprecated)]
            EdidR4DisplayRangeVideoTimingsSupport::SecondaryGTF(g) => {
                let blank_grad_lo = (g.blanking_gradient & 0xff) as u8;
                let blank_grad_hi = (g.blanking_gradient >> 8) as u8;

                bytes.extend_from_slice(&[
                    0x02,
                    0x00,
                    g.horizontal_start_frequency.into_raw(),
                    g.blanking_offset * 2,
                    blank_grad_lo,
                    blank_grad_hi,
                    g.blanking_scaling_factor,
                    g.blanking_scaling_factor_weighting * 2,
                ]);
            }
            EdidR4DisplayRangeVideoTimingsSupport::CVTSupported(v) => {
                match v {
                    EdidR4DisplayRangeVideoTimingsCVT::R1(cvt) => {
                        bytes.extend_from_slice(&[0x04, 0x11]);

                        // FIXME: Check that it fits in 6 bits.
                        let pclk_diff = EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff::try_from(
                            self.max_pixelclock,
                        )
                        .expect("Pixel Clock value would overflow our type")
                        .into_raw();
                        let raw_max_pix = round_up(cvt.maximum_active_pixels_per_line, 8) / 8;
                        let max_pix_hi = ((raw_max_pix >> 8) & 0x3) as u8;
                        let max_pix_lo = (raw_max_pix & 0xff) as u8;

                        bytes.extend_from_slice(&[(pclk_diff << 2) | max_pix_hi, max_pix_lo]);

                        let mut byte: u8 = 0;
                        for ratio in cvt.supported_aspect_ratios {
                            byte |= 1 << ((ratio as u8) + 3);
                        }
                        bytes.push(byte);

                        let mut byte = (cvt.preferred_aspect_ratio as u8) << 5;
                        if cvt.reduced_cvt_blanking_supported {
                            byte |= 1 << 4;
                        }

                        if cvt.standard_cvt_blanking_supported {
                            byte |= 1 << 3;
                        }
                        bytes.push(byte);

                        let mut byte = 0;
                        if cvt.horizontal_shrink_supported {
                            byte |= 1 << 7;
                        }

                        if cvt.horizontal_stretch_supported {
                            byte |= 1 << 6;
                        }

                        if cvt.vertical_shrink_supported {
                            byte |= 1 << 5;
                        }

                        if cvt.vertical_stretch_supported {
                            byte |= 1 << 4;
                        }
                        bytes.push(byte);
                        bytes.push(cvt.preferred_vertical_refresh_rate.0);
                    }
                }
            }
        };

        bytes
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub enum EdidR4DescriptorEstablishedTimingsIII {
    ET_1152_864_75Hz = 0,
    ET_1024_768_85Hz,
    ET_800_600_85Hz,
    ET_848_480_60Hz,
    ET_640_480_85Hz,
    ET_720_400_85Hz,
    ET_640_400_85Hz,
    ET_640_350_85Hz,
    ET_1280_1024_85Hz,
    ET_1280_1024_60Hz,
    ET_1280_960_85Hz,
    ET_1280_960_60Hz,
    ET_1280_768_85Hz,
    ET_1280_768_75Hz,
    ET_1280_768_60Hz,
    ET_1280_768_60Hz_RB,
    ET_1400_1050_75Hz,
    ET_1400_1050_60Hz,
    ET_1400_1050_60Hz_RB,
    ET_1440_900_85Hz,
    ET_1440_900_75Hz,
    ET_1440_900_60Hz,
    ET_1440_900_60Hz_RB,
    ET_1360_768_60Hz,
    ET_1600_1200_70Hz,
    ET_1600_1200_65Hz,
    ET_1600_1200_60Hz,
    ET_1680_1050_85Hz,
    ET_1680_1050_75Hz,
    ET_1680_1050_60Hz,
    ET_1680_1050_60Hz_RB,
    ET_1400_1050_85Hz,
    ET_1920_1200_60Hz,
    ET_1920_1200_60Hz_RB,
    ET_1856_1392_75Hz,
    ET_1856_1392_60Hz,
    ET_1792_1344_75Hz,
    ET_1792_1344_60Hz,
    ET_1600_1200_85Hz,
    ET_1600_1200_75Hz,

    ET_1920_1440_75Hz = 44,
    ET_1920_1440_60Hz,
    ET_1920_1200_85Hz,
    ET_1920_1200_75Hz,
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(mutators(
    pub fn established_timings(&mut self, et: Vec<EdidR4DescriptorEstablishedTimingsIII>) {
        self.established_timings = et;
    }

    pub fn add_established_timing(&mut self, et: EdidR4DescriptorEstablishedTimingsIII) {
        self.established_timings.push(et);
    }
))]
pub struct EdidR4DescriptorEstablishedTimings {
    #[builder(via_mutators)]
    established_timings: Vec<EdidR4DescriptorEstablishedTimingsIII>,
}

impl IntoBytes for EdidR4DescriptorEstablishedTimings {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_PAYLOAD_LEN);
        bytes.push(0x0a);

        let mut array: [u8; 6] = [0; 6];
        for timing in self.established_timings {
            let id = timing as u32;
            let idx = (id / 8) as usize;
            let shift = id % 8;

            array[idx] |= 1 << shift;
        }

        bytes.extend_from_slice(&array);
        bytes.extend_from_slice(&[0; 6]);

        bytes
    }
}

#[derive(Clone, Debug)]
pub enum EdidR3Descriptor {
    DetailedTiming(EdidDescriptorDetailedTiming),
    Custom(EdidDescriptorCustom),
    Dummy,
    StandardTimings(()),
    ColorPointData(()),
    ProductName(EdidDescriptorString),
    DisplayRangeLimits(EdidR3DisplayRangeLimits),
    DataString(EdidDescriptorString),
    ProductSerialNumber(EdidDescriptorString),
}

impl IntoBytes for EdidR3Descriptor {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            Self::DetailedTiming(dtd) => dtd.into_bytes(),
            Self::Custom(c) => c.into_bytes(),
            Self::Dummy => Vec::from(&[0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            Self::StandardTimings(()) => unimplemented!(),
            Self::ColorPointData(()) => unimplemented!(),
            Self::ProductName(v) => {
                let mut bytes: Vec<u8> = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

                bytes.extend_from_slice(&[0, 0, 0, 0xfc, 0]);
                bytes.extend_from_slice(&v.into_bytes());

                bytes
            }
            Self::DisplayRangeLimits(drl) => {
                let mut bytes: Vec<u8> = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

                bytes.extend_from_slice(&[0, 0, 0, 0xfd, 0]);
                bytes.extend_from_slice(&drl.into_bytes());

                bytes
            }
            Self::DataString(v) => {
                let mut bytes: Vec<u8> = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

                bytes.extend_from_slice(&[0, 0, 0, 0xfe, 0]);
                bytes.extend_from_slice(&v.into_bytes());

                bytes
            }
            Self::ProductSerialNumber(v) => {
                let mut bytes: Vec<u8> = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

                bytes.extend_from_slice(&[0, 0, 0, 0xff, 0]);
                bytes.extend_from_slice(&v.into_bytes());

                bytes
            }
        };

        assert_eq!(
            bytes.len(),
            EDID_DESCRIPTOR_LEN,
            "Descriptor Size has a different size than it should ({} vs expected {})",
            bytes.len(),
            EDID_DESCRIPTOR_LEN
        );

        bytes
    }
}

#[derive(Clone, Debug)]
pub enum EdidR4Descriptor {
    DetailedTiming(EdidDescriptorDetailedTiming),
    Custom(EdidDescriptorCustom),
    Dummy,
    EstablishedTimings(EdidR4DescriptorEstablishedTimings),
    CVT(()),
    DisplayColorManagement(()),
    StandardTimings(()),
    ColorPointData(()),
    ProductName(EdidDescriptorString),
    DisplayRangeLimits(EdidR4DisplayRangeLimits),
    DataString(EdidDescriptorString),
    ProductSerialNumber(EdidDescriptorString),
}

impl IntoBytes for EdidR4Descriptor {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            Self::DetailedTiming(dtd) => dtd.into_bytes(),
            Self::Custom(c) => EdidR3Descriptor::Custom(c).into_bytes(),
            Self::Dummy => EdidR3Descriptor::Dummy.into_bytes(),
            Self::EstablishedTimings(et) => {
                let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

                bytes.extend_from_slice(&[0, 0, 0, 0xf7, 0]);
                bytes.extend_from_slice(&et.into_bytes());

                bytes
            }
            Self::CVT(()) => unimplemented!(),
            Self::DisplayColorManagement(()) => unimplemented!(),
            Self::StandardTimings(()) => unimplemented!(),
            Self::ColorPointData(()) => unimplemented!(),
            Self::ProductName(v) => EdidR3Descriptor::ProductName(v).into_bytes(),
            Self::DisplayRangeLimits(drl) => {
                let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

                bytes.extend_from_slice(&[0, 0, 0, 0xfd]);
                bytes.extend_from_slice(&drl.into_bytes());

                bytes
            }
            Self::DataString(v) => EdidR3Descriptor::DataString(v).into_bytes(),
            Self::ProductSerialNumber(v) => EdidR3Descriptor::ProductSerialNumber(v).into_bytes(),
        };

        assert_eq!(
            bytes.len(),
            EDID_DESCRIPTOR_LEN,
            "Descriptor Size has a different size than it should ({} vs expected {})",
            bytes.len(),
            EDID_DESCRIPTOR_LEN
        );

        bytes
    }
}

#[cfg(test)]
mod tests {
    use crate::{EdidR4Descriptor, IntoBytes};
    use std::convert::TryInto;

    #[test]
    fn test_descriptor_product_name_spec() {
        // NOTE: There's a typo in the EDID 1.4 Spec (Release A, Rev2) where the XYZ Monitor is
        // supposed to translate to what is actually XYZ Monltor
        assert_eq!(
            EdidR4Descriptor::ProductName("XYZ Monitor".try_into().unwrap()).into_bytes(),
            [
                0x00, 0x00, 0x00, 0xfc, 0x00, 0x58, 0x59, 0x5a, 0x20, 0x4d, 0x6f, 0x6e, 0x69, 0x74,
                0x6f, 0x72, 0x0a, 0x20
            ]
        );
    }

    #[test]
    fn test_descriptor_data_string_spec() {
        assert_eq!(
            EdidR4Descriptor::DataString("THISISATEST".try_into().unwrap()).into_bytes(),
            [
                0x00, 0x00, 0x00, 0xfe, 0x00, 0x54, 0x48, 0x49, 0x53, 0x49, 0x53, 0x41, 0x54, 0x45,
                0x53, 0x54, 0x0A, 0x20
            ]
        );
    }

    #[test]
    fn test_descriptor_product_serial_number_spec() {
        assert_eq!(
            EdidR4Descriptor::ProductSerialNumber("A0123456789".try_into().unwrap()).into_bytes(),
            [
                0x00, 0x00, 0x00, 0xff, 0x00, 0x41, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
                0x38, 0x39, 0x0A, 0x20,
            ]
        );
    }
}

#[derive(Clone, Debug)]
pub enum EdidDescriptor {
    R3(EdidR3Descriptor),
    R4(EdidR4Descriptor),
}

impl IntoBytes for Vec<EdidDescriptor> {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTORS_NUM * EDID_DESCRIPTOR_LEN);

        let mut last_idx = 0;
        for (idx, desc) in self.into_iter().enumerate() {
            let desc_bytes = match desc {
                EdidDescriptor::R3(e) => e.into_bytes(),
                EdidDescriptor::R4(e) => e.into_bytes(),
            };

            last_idx = idx;
            bytes.extend_from_slice(&desc_bytes);
        }

        for _ in (last_idx + 1)..EDID_DESCRIPTORS_NUM {
            bytes.extend_from_slice(&EdidR3Descriptor::Dummy.into_bytes());
        }

        assert_eq!(
            bytes.len(),
            EDID_DESCRIPTOR_LEN * EDID_DESCRIPTORS_NUM,
            "Descriptor Size has a different size than it should ({} vs expected {})",
            bytes.len(),
            EDID_DESCRIPTOR_LEN * EDID_DESCRIPTORS_NUM
        );

        bytes
    }
}
