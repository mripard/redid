use core::{cmp, fmt, num::NonZero, ops::Range};

use encoding::{all::ISO_8859_1, EncoderTrap, Encoding as _};
use num_traits::{Bounded, CheckedShl, Num, ToPrimitive as _, WrappingSub};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer};
use typed_builder::TypedBuilder;

use crate::{
    EdidTypeConversionError, IntoBytes, EDID_DESCRIPTORS_NUM, EDID_DESCRIPTOR_LEN,
    EDID_DESCRIPTOR_PAYLOAD_LEN,
};

fn compute_max_value<T>(num_bits: usize) -> T
where
    T: Num + Bounded + CheckedShl + WrappingSub,
{
    let type_num_bits = size_of::<T>() * 8;

    assert!(
        num_bits <= type_num_bits,
        "Number of bits is greater than can be stored in the type"
    );

    match num_bits.cmp(&type_num_bits) {
        cmp::Ordering::Less => {
            // Thanks to the assert above, we know that num_bits is going to be at most 128, which
            // fits into a u32 with plenty of headroom.
            #[allow(clippy::unwrap_used)]
            let rhs = num_bits.to_u32().unwrap();

            // checked_shl returns None if rhs is equal to or larger than the number of bits in T.
            // However, we're in that branch only if the number of bits we want to shift of is
            // strictly lower than the number of bits in our type, so we'll always get a value.
            #[allow(clippy::unwrap_used)]
            let shl = T::checked_shl(&T::one(), rhs).unwrap();
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8"))]
pub struct EdidDescriptorCustomTag(u8);

impl TryFrom<u8> for EdidDescriptorCustomTag {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 0x0f {
            Err(EdidTypeConversionError::Range(value, Some(0), Some(0x0f)))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "Vec<u8>"))]
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

/// Descriptor defined by manufacturer.
/// Defined in EDID 1.4 Specification, Section 3.10.3.12.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDescriptorCustom {
    /// Data tag numbers from 0x00 to 0x0F
    tag: EdidDescriptorCustomTag,
    /// Vendor-specific data
    payload: EdidDescriptorCustomPayload,
}

impl IntoBytes for EdidDescriptorCustom {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(EDID_DESCRIPTOR_LEN);

        let tag = self.tag.0;
        bytes.extend_from_slice(&[0, 0, 0, tag, 0]);
        bytes.extend_from_slice(&self.payload.0);
        bytes.resize(EDID_DESCRIPTOR_LEN, 0);

        let len = bytes.len();
        assert_eq!(
            len, EDID_DESCRIPTOR_LEN,
            "Custom Descriptor is too large ({len} vs expected {EDID_DESCRIPTOR_PAYLOAD_LEN})",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_LEN
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
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

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_PAYLOAD_LEN
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u32"))]
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
                Some(655_350),
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidDetailedTimingAnalogSync {
    BipolarComposite(bool, bool),
    Composite(bool, bool),
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDetailedTimingDigitalCompositeSync {
    #[builder(default)]
    serrations: bool,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDetailedTimingDigitalSeparateSync {
    #[builder(default)]
    vsync_positive: bool,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidDetailedTimingDigitalSyncKind {
    Composite(EdidDetailedTimingDigitalCompositeSync),
    Separate(EdidDetailedTimingDigitalSeparateSync),
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDetailedTimingDigitalSync {
    kind: EdidDetailedTimingDigitalSyncKind,

    #[builder(default)]
    hsync_positive: bool,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidDetailedTimingSync {
    Analog(EdidDetailedTimingAnalogSync),
    Digital(EdidDetailedTimingDigitalSync),
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
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
pub struct EdidDescriptorTiming<const N: usize, T: fmt::Display>(T);

impl<const N: usize, T> Default for EdidDescriptorTiming<N, T>
where
    T: Num + fmt::Display,
{
    fn default() -> Self {
        Self(T::zero())
    }
}

impl<const N: usize, T> EdidDescriptorTiming<N, T>
where
    T: Num + Bounded + CheckedShl + WrappingSub + PartialOrd + fmt::Display,
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

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8"))]
pub struct EdidDescriptor6BitsTiming(EdidDescriptorTiming<6, u8>);

impl EdidDescriptor6BitsTiming {
    fn into_raw(self) -> u8 {
        self.0.into_raw()
    }
}

impl TryFrom<u8> for EdidDescriptor6BitsTiming {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(EdidDescriptorTiming::try_from(value)?))
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

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8"))]
pub struct EdidDescriptor8BitsTiming(EdidDescriptorTiming<8, u8>);

impl EdidDescriptor8BitsTiming {
    fn into_raw(self) -> u8 {
        self.0.into_raw()
    }
}

impl TryFrom<u8> for EdidDescriptor8BitsTiming {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(EdidDescriptorTiming::try_from(value)?))
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

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
pub struct EdidDescriptor10BitsTiming(EdidDescriptorTiming<10, u16>);

impl EdidDescriptor10BitsTiming {
    fn into_raw(self) -> u16 {
        self.0.into_raw()
    }
}

impl TryFrom<u16> for EdidDescriptor10BitsTiming {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(EdidDescriptorTiming::try_from(value)?))
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

#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
pub struct EdidDescriptor12BitsTiming(EdidDescriptorTiming<12, u16>);

impl EdidDescriptor12BitsTiming {
    fn into_raw(self) -> u16 {
        self.0.into_raw()
    }
}

impl TryFrom<u16> for EdidDescriptor12BitsTiming {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(EdidDescriptorTiming::try_from(value)?))
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDescriptorDetailedTimingHorizontal {
    active: EdidDescriptor12BitsTiming,
    front_porch: EdidDescriptor10BitsTiming,
    sync_pulse: EdidDescriptor10BitsTiming,
    back_porch: EdidDescriptor12BitsTiming,
    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    border: EdidDescriptor8BitsTiming,
    size_mm: EdidDetailedTimingSizeMm,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDescriptorDetailedTimingVertical {
    active: EdidDescriptor12BitsTiming,
    front_porch: EdidDescriptor6BitsTiming,
    sync_pulse: EdidDescriptor6BitsTiming,
    back_porch: EdidDescriptor12BitsTiming,
    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    border: EdidDescriptor8BitsTiming,
    size_mm: EdidDetailedTimingSizeMm,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDescriptorDetailedTiming {
    pixel_clock: EdidDetailedTimingPixelClock,

    horizontal: EdidDescriptorDetailedTimingHorizontal,
    vertical: EdidDescriptorDetailedTimingVertical,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    interlace: bool,

    sync_type: EdidDetailedTimingSync,
    stereo: EdidDetailedTimingStereo,
}

impl IntoBytes for EdidDescriptorDetailedTiming {
    #[allow(clippy::too_many_lines)]
    fn into_bytes(self) -> Vec<u8> {
        let freq = self.pixel_clock.into_raw();
        let freq_lo = (freq & 0xff) as u8;
        let freq_hi = ((freq >> 8) & 0xff) as u8;

        let hact = self.horizontal.active.into_raw();
        let hact_lo = (hact & 0xff) as u8;
        let hact_hi = ((hact >> 8) & 0xf) as u8;

        let hborder = self.horizontal.border.into_raw();
        let hfp = self.horizontal.front_porch.into_raw();
        let hso = u16::from(hborder) + hfp;
        let _: EdidDescriptor10BitsTiming = EdidDescriptor10BitsTiming::try_from(hso)
            .expect("Horizontal Front Porch and Border don't fit into 10 bits.");

        let hso_lo = (hso & 0xff) as u8;
        let hso_hi = ((hso >> 8) & 0x3) as u8;

        let hsync = self.horizontal.sync_pulse.into_raw();
        let hsync_lo = (hsync & 0xff) as u8;
        let hsync_hi = ((hsync >> 8) & 0x3) as u8;

        let hbp = self.horizontal.back_porch.into_raw();
        let hblank = hso + hsync + hbp + u16::from(hborder);
        let _: EdidDescriptor12BitsTiming = EdidDescriptor12BitsTiming::try_from(hblank).expect(
            "Horizontal Front Porch, Back Porch, Sync Pulse and Borders don't fit into 12 bits.",
        );

        let hblank_lo = (hblank & 0xff) as u8;
        let hblank_hi = ((hblank >> 8) & 0xf) as u8;

        let vact = self.vertical.active.into_raw();
        let vact_lo = (vact & 0xff) as u8;
        let vact_hi = ((vact >> 8) & 0xf) as u8;

        let vborder = self.vertical.border.into_raw();
        let vfp = self.vertical.front_porch.into_raw();
        let vso = vborder + vfp;
        let _: EdidDescriptor6BitsTiming = EdidDescriptor6BitsTiming::try_from(vso)
            .expect("Vertical Front Porch and Border don't fit into 6 bits.");

        let vso_lo = vso & 0xf;
        let vso_hi = (vso >> 4) & 0x3;

        let vsync = self.vertical.sync_pulse.into_raw();
        let vsync_lo = vsync & 0xf;
        let vsync_hi = (vsync >> 4) & 0x3;

        let vbp = self.vertical.back_porch.into_raw();
        let vblank = u16::from(vso) + u16::from(vsync) + vbp + u16::from(vborder);
        let _: EdidDescriptor12BitsTiming = EdidDescriptor12BitsTiming::try_from(vblank).expect(
            "Horizontal Front Porch, Back Porch, Sync Pulse and Borders don't fit into 12 bits.",
        );

        let vblank_lo = (vblank & 0xff) as u8;
        let vblank_hi = ((vblank >> 8) & 0xf) as u8;

        let hsize = self.horizontal.size_mm.into_raw();
        let hsize_lo = (hsize & 0xff) as u8;
        let hsize_hi = ((hsize >> 8) & 0xf) as u8;

        let vsize = self.vertical.size_mm.into_raw();
        let vsize_lo = (vsize & 0xff) as u8;
        let vsize_hi = ((vsize >> 8) & 0xf) as u8;

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
                }

                if v.hsync_positive {
                    flags |= 1 << 1;
                }
            }
        }

        let data: [u8; EDID_DESCRIPTOR_LEN] = [
            freq_lo,
            freq_hi,
            hact_lo,
            hblank_lo,
            (hact_hi << 4) | hblank_hi,
            vact_lo,
            vblank_lo,
            (vact_hi << 4) | vblank_hi,
            hso_lo,
            hsync_lo,
            (vso_lo << 4) | vsync_lo,
            (hso_hi << 6) | (hsync_hi << 4) | (vso_hi << 2) | vsync_hi,
            hsize_lo,
            vsize_lo,
            (hsize_hi << 4) | vsize_hi,
            hborder,
            vborder,
            flags,
        ];

        data.to_vec()
    }

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_LEN
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8"))]
pub struct EdidDisplayRangeLimitsFrequency(NonZero<u8>);

impl TryFrom<u8> for EdidDisplayRangeLimitsFrequency {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(NonZero::new(value).ok_or(
            EdidTypeConversionError::Value(String::from("Frequency cannot be 0.")),
        )?))
    }
}

#[derive(Clone, Debug)]
pub struct EdidDisplayRangeLimitsRangeFreq(Range<EdidDisplayRangeLimitsFrequency>);

impl TryFrom<Range<EdidDisplayRangeLimitsFrequency>> for EdidDisplayRangeLimitsRangeFreq {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: Range<EdidDisplayRangeLimitsFrequency>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(EdidTypeConversionError::Value(format!(
                "Empty Range: ({}..{})",
                value.start.0, value.end.0
            )));
        }

        Ok(Self(Range {
            start: value.start,
            end: value.end,
        }))
    }
}

impl TryFrom<Range<u8>> for EdidDisplayRangeLimitsRangeFreq {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: Range<u8>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(EdidTypeConversionError::Value(format!(
                "Empty Range: ({}..{})",
                value.start, value.end
            )));
        }

        Ok(Self(Range {
            start: value.start.try_into()?,
            end: value.end.try_into()?,
        }))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EdidDisplayRangeLimitsRangeFreq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RangeHelper {
            min: u8,
            max: u8,
        }

        let helper = RangeHelper::deserialize(deserializer)?;
        Self::try_from(helper.min..helper.max).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
pub struct EdidDisplayRangePixelClock(u16);

impl EdidDisplayRangePixelClock {
    fn round(self) -> u16 {
        self.0.next_multiple_of(10)
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidDisplayRangeVideoTimingsGTF {
    #[builder(setter(into))]
    horizontal_start_frequency: EdidDisplayRangeVideoTimingsGTFStartFrequency,
    blanking_offset: u8,
    blanking_gradient: u16,
    blanking_scaling_factor: u8,
    blanking_scaling_factor_weighting: u8,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum EdidR3DisplayRangeVideoTimingsSupport {
    #[cfg_attr(feature = "serde", serde(rename = "default_gtf"))]
    DefaultGTF,
    #[cfg_attr(feature = "serde", serde(rename = "secondary_gtf"))]
    SecondaryGTF(EdidDisplayRangeVideoTimingsGTF),
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidR3DisplayRangeLimits {
    hfreq_khz: EdidDisplayRangeLimitsRangeFreq,
    vfreq_hz: EdidDisplayRangeLimitsRangeFreq,
    max_pixelclock_mhz: EdidDisplayRangePixelClock,

    timings_support: EdidR3DisplayRangeVideoTimingsSupport,
}

impl IntoBytes for EdidR3DisplayRangeLimits {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_PAYLOAD_LEN);

        bytes.push(self.vfreq_hz.0.start.0.get());
        bytes.push(self.vfreq_hz.0.end.0.get());
        bytes.push(self.hfreq_khz.0.start.0.get());
        bytes.push(self.hfreq_khz.0.end.0.get());
        bytes.push(self.max_pixelclock_mhz.into_raw());

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
        }

        let len = bytes.len();
        assert_eq!(
            len, EDID_DESCRIPTOR_PAYLOAD_LEN,
            "Descriptor Payload is larger than it should ({len} vs expected {EDID_DESCRIPTOR_PAYLOAD_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_PAYLOAD_LEN
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
pub struct EdidR4DisplayRangeLimitsFrequency(u16);

impl TryFrom<u16> for EdidR4DisplayRangeLimitsFrequency {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == 0 {
            return Err(EdidTypeConversionError::Value(String::from(
                "Frequency cannot be 0.",
            )));
        }

        if value > 510 {
            return Err(EdidTypeConversionError::Value(String::from(
                "Frequency cannot be higher than 510.",
            )));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Debug)]
pub struct EdidR4DisplayRangeLimitsRangeFreq(Range<EdidR4DisplayRangeLimitsFrequency>);

impl TryFrom<Range<EdidR4DisplayRangeLimitsFrequency>> for EdidR4DisplayRangeLimitsRangeFreq {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: Range<EdidR4DisplayRangeLimitsFrequency>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(EdidTypeConversionError::Value(format!(
                "Empty Range: ({}..{})",
                value.start.0, value.end.0
            )));
        }

        Ok(Self(Range {
            start: value.start,
            end: value.end,
        }))
    }
}

impl TryFrom<Range<u16>> for EdidR4DisplayRangeLimitsRangeFreq {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: Range<u16>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(EdidTypeConversionError::Value(format!(
                "Empty Range: ({}..{})",
                value.start, value.end
            )));
        }

        Ok(Self(Range {
            start: value.start.try_into()?,
            end: value.end.try_into()?,
        }))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EdidR4DisplayRangeLimitsRangeFreq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RangeHelper {
            min: u16,
            max: u16,
        }

        let helper = RangeHelper::deserialize(deserializer)?;
        Self::try_from(helper.min..helper.max).map_err(de::Error::custom)
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidR4DisplayRangeVideoTimingsAspectRatio {
    Ratio_4_3 = 0,
    Ratio_16_9,
    Ratio_16_10,
    Ratio_5_4,
    Ratio_15_9,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
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

impl TryFrom<u16> for EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let clock = EdidDisplayRangePixelClock::try_from(value)?;

        match EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff::try_from(clock) {
            Ok(v) => Ok(v),
            Err(e) => match e {
                EdidTypeConversionError::Int(ie) => Err(Self::Error::Int(ie)),
                EdidTypeConversionError::Slice(se) => Err(Self::Error::Slice(se)),
                EdidTypeConversionError::Range(a, b, c) => Err(Self::Error::Range(
                    a.into(),
                    b.map(Into::into),
                    c.map(Into::into),
                )),
                EdidTypeConversionError::Value(v) => Err(Self::Error::Value(v)),
            },
        }
    }
}

impl EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff {
    fn into_raw(self) -> u8 {
        self.0 * 4
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn supported_aspect_ratios(&mut self, ar: Vec<EdidR4DisplayRangeVideoTimingsAspectRatio>) {
        self.supported_aspect_ratios = ar;
    }

    #[allow(unreachable_pub)]
    pub fn add_supported_aspect_ratio(&mut self, ar: EdidR4DisplayRangeVideoTimingsAspectRatio) {
        self.supported_aspect_ratios.push(ar);
    }
))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidR4DisplayRangeVideoTimingsCVTR1 {
    // FIXME: Max 8184 pixels
    maximum_active_pixels_per_line: u16,
    #[builder(via_mutators)]
    supported_aspect_ratios: Vec<EdidR4DisplayRangeVideoTimingsAspectRatio>,
    preferred_aspect_ratio: EdidR4DisplayRangeVideoTimingsAspectRatio,

    #[builder(default)]
    standard_cvt_blanking_supported: bool,

    #[builder(default)]
    reduced_cvt_blanking_supported: bool,

    #[builder(default)]
    horizontal_shrink_supported: bool,

    #[builder(default)]
    horizontal_stretch_supported: bool,

    #[builder(default)]
    vertical_shrink_supported: bool,

    #[builder(default)]
    vertical_stretch_supported: bool,

    #[builder(setter(into))]
    preferred_vertical_refresh_rate: EdidDisplayRangeLimitsFrequency,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidR4DisplayRangeVideoTimingsCVT {
    R1(EdidR4DisplayRangeVideoTimingsCVTR1),
}

// It's a bit of an ugly workaround, but otherwise the serde Deserialize implementation will
// trigger a deprecation warning.
//
// See https://github.com/rust-lang/rust/issues/87454
#[allow(deprecated)]
mod vid_timing {
    use super::{EdidDisplayRangeVideoTimingsGTF, EdidR4DisplayRangeVideoTimingsCVT};
    #[cfg(feature = "serde")]
    use serde::Deserialize;

    #[derive(Clone, Debug)]
    #[cfg_attr(feature = "serde", derive(Deserialize))]
    #[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
    pub enum EdidR4DisplayRangeVideoTimingsSupport {
        #[cfg_attr(feature = "serde", serde(rename = "default_gtf"))]
        DefaultGTF,
        RangeLimitsOnly,
        #[deprecated = "GTF is considered obsolete with EDID 1.4"]
        SecondaryGTF(EdidDisplayRangeVideoTimingsGTF),
        #[cfg_attr(feature = "serde", serde(rename = "cvt_supported"))]
        CVTSupported(EdidR4DisplayRangeVideoTimingsCVT),
    }
}
pub use vid_timing::EdidR4DisplayRangeVideoTimingsSupport;

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidR4DisplayRangeLimits {
    #[builder(setter(into))]
    hfreq_khz: EdidR4DisplayRangeLimitsRangeFreq,

    #[builder(setter(into))]
    vfreq_hz: EdidR4DisplayRangeLimitsRangeFreq,

    #[builder(setter(into))]
    max_pixelclock_mhz: EdidDisplayRangePixelClock,

    timings_support: EdidR4DisplayRangeVideoTimingsSupport,
}

impl IntoBytes for EdidR4DisplayRangeLimits {
    fn into_bytes(self) -> Vec<u8> {
        // The Display Range Limits block has a header a byte shorter than other descriptors.
        let mut bytes = Vec::with_capacity(EDID_DESCRIPTOR_PAYLOAD_LEN + 1);

        let mut flags_byte = 0;

        let [min_hfreq_hi, min_hfreq_lo] = self.hfreq_khz.0.start.0.to_be_bytes();
        let [max_hfreq_hi, max_hfreq_lo] = self.hfreq_khz.0.end.0.to_be_bytes();
        let [min_vfreq_hi, min_vfreq_lo] = self.vfreq_hz.0.start.0.to_be_bytes();
        let [max_vfreq_hi, max_vfreq_lo] = self.vfreq_hz.0.end.0.to_be_bytes();

        if max_vfreq_hi > 0 {
            flags_byte |= 1 << 1;

            if min_vfreq_hi > 0 {
                flags_byte |= 1 << 0;
            }
        }

        if max_hfreq_hi > 0 {
            flags_byte |= 1 << 3;

            if min_hfreq_hi > 0 {
                flags_byte |= 1 << 2;
            }
        }

        bytes.push(flags_byte);
        bytes.push(min_vfreq_lo);
        bytes.push(max_vfreq_lo);
        bytes.push(min_hfreq_lo);
        bytes.push(max_hfreq_lo);
        bytes.push(self.max_pixelclock_mhz.into_raw());

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
                            self.max_pixelclock_mhz,
                        )
                        .expect("Pixel Clock value would overflow our type")
                        .into_raw();
                        let raw_max_pix = cvt.maximum_active_pixels_per_line.div_ceil(8);
                        let max_pix_hi = ((raw_max_pix >> 8) & 0x3) as u8;
                        let max_pix_lo = (raw_max_pix & 0xff) as u8;

                        bytes.extend_from_slice(&[(pclk_diff << 2) | max_pix_hi, max_pix_lo]);

                        let mut byte: u8 = 0;
                        for ratio in cvt.supported_aspect_ratios {
                            byte |= 1 << (7 - (ratio as u8));
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
                        bytes.push(cvt.preferred_vertical_refresh_rate.0.get());
                    }
                }
            }
        }

        let len = bytes.len();
        assert_eq!(
            len,
            EDID_DESCRIPTOR_PAYLOAD_LEN + 1,
            "Descriptor Payload is larger than it should ({len} vs expected {} bytes)",
            EDID_DESCRIPTOR_PAYLOAD_LEN + 1
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_PAYLOAD_LEN + 1
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
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
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn established_timings(&mut self, et: Vec<EdidR4DescriptorEstablishedTimingsIII>) {
        self.established_timings = et;
    }

    #[allow(unreachable_pub)]
    pub fn add_established_timing(&mut self, et: EdidR4DescriptorEstablishedTimingsIII) {
        self.established_timings.push(et);
    }
))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
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

        let len = bytes.len();
        assert_eq!(
            len, EDID_DESCRIPTOR_PAYLOAD_LEN,
            "Descriptor Payload is larger than it should ({len} vs expected {EDID_DESCRIPTOR_PAYLOAD_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_PAYLOAD_LEN
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
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

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_LEN
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
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

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_LEN
    }
}

#[cfg(test)]
mod tests {
    use crate::{EdidDescriptorCustom, EdidR4Descriptor, IntoBytes};

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

    #[test]
    fn test_descriptor_custom_valid() {
        let tag: u8 = 0x00;
        let payload: Vec<u8> = vec![0xED, 0xD1, 0xD0, 0x00];

        assert_eq!(
            EdidR4Descriptor::Custom((tag, payload).try_into().unwrap()).into_bytes(),
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0xED, 0xD1, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ]
        );
    }

    #[test]
    fn test_descriptor_custom_invalid_tag() {
        let tag: u8 = 0x10;
        let payload: Vec<u8> = vec![0xC0, 0xFF, 0xEE];

        assert_eq!(
            EdidDescriptorCustom::try_from((tag, payload))
                .expect_err("Should fail")
                .to_string(),
            "Value out of range: 16 (Range: 0..=15)"
        );
    }

    #[test]
    fn test_descriptor_custom_payload_too_long() {
        let tag: u8 = 0x01;
        let payload: Vec<u8> = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        ];

        assert_eq!(
            EdidDescriptorCustom::try_from((tag, payload))
                .expect_err("Should fail")
                .to_string(),
            "Invalid Value: Custom Descriptor Payload must be at most 13 bytes long."
        );
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
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

    fn size(&self) -> usize {
        EDID_DESCRIPTOR_LEN * EDID_DESCRIPTORS_NUM
    }
}
