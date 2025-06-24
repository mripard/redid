// Copyright 2020-2024, Maxime Ripard
// Licensed under the MIT License
// See the LICENSE file or <http://opensource.org/licenses/MIT>

// FIXME: Write all the doc
#![allow(missing_docs)]
#![expect(
    clippy::doc_nested_refdefs,
    reason = "Markdown Checklists seem to confuse clippy"
)]
#![expect(
    clippy::similar_names,
    reason = "We do have similar names. It's kind of expected."
)]
#![expect(
    clippy::struct_excessive_bools,
    reason = "We do indeed have structures with plenty of bools. There's not much we can do about it."
)]
#![doc = include_str!("../README.md")]

use core::{array, fmt, num};

use num_traits::ToPrimitive as _;
use static_assertions::const_assert_eq;
use typed_builder::TypedBuilder;

mod descriptors;

pub use descriptors::{
    EdidDescriptor, EdidDescriptor10BitsTiming, EdidDescriptor12BitsTiming,
    EdidDescriptor6BitsTiming, EdidDescriptor8BitsTiming, EdidDescriptorCustom,
    EdidDescriptorCustomPayload, EdidDescriptorCustomTag, EdidDescriptorDetailedTiming,
    EdidDescriptorString, EdidDescriptorTiming, EdidDetailedTimingAnalogSync,
    EdidDetailedTimingDigitalCompositeSync, EdidDetailedTimingDigitalSeparateSync,
    EdidDetailedTimingDigitalSync, EdidDetailedTimingDigitalSyncKind, EdidDetailedTimingPixelClock,
    EdidDetailedTimingSizeMm, EdidDetailedTimingStereo, EdidDetailedTimingSync,
    EdidDisplayRangeHorizontalFreq, EdidDisplayRangePixelClock, EdidDisplayRangeVerticalFreq,
    EdidDisplayRangeVideoTimingsGTF, EdidDisplayRangeVideoTimingsGTFStartFrequency,
    EdidR3Descriptor, EdidR3DisplayRangeLimits, EdidR3DisplayRangeVideoTimingsSupport,
    EdidR4Descriptor, EdidR4DescriptorEstablishedTimings, EdidR4DescriptorEstablishedTimingsIII,
    EdidR4DisplayRangeHorizontalFreq, EdidR4DisplayRangeLimits, EdidR4DisplayRangeVerticalFreq,
    EdidR4DisplayRangeVideoTimingsAspectRatio, EdidR4DisplayRangeVideoTimingsCVT,
    EdidR4DisplayRangeVideoTimingsCVTPixelClockDiff, EdidR4DisplayRangeVideoTimingsCVTR1,
    EdidR4DisplayRangeVideoTimingsSupport,
};

mod extensions;

pub use extensions::{
    CecAddress, EdidExtension, EdidExtensionCTA861, EdidExtensionCTA861AudioDataBlock,
    EdidExtensionCTA861AudioDataBlockChannels, EdidExtensionCTA861AudioDataBlockDesc,
    EdidExtensionCTA861AudioDataBlockLPCM, EdidExtensionCTA861AudioDataBlockSamplingFrequency,
    EdidExtensionCTA861AudioDataBlockSamplingRate, EdidExtensionCTA861ColorimetryDataBlock,
    EdidExtensionCTA861Hdmi14bDataBlockVideo, EdidExtensionCTA861Hdmi14bTmdsRate,
    EdidExtensionCTA861HdmiDataBlock, EdidExtensionCTA861Revision3,
    EdidExtensionCTA861Revision3DataBlock, EdidExtensionCTA861SpeakerAllocationDataBlock,
    EdidExtensionCTA861VideoCapabilityDataBlock, EdidExtensionCTA861VideoCapabilityQuantization,
    EdidExtensionCTA861VideoCapabilityScanBehavior, EdidExtensionCTA861VideoDataBlock,
    EdidExtensionCTA861VideoDataBlockDesc,
};

mod utils;

const EDID_BASE_LEN: usize = 128;

// It looks like const_assert! doesn't count as being used somehow.
#[allow(dead_code)]
const EDID_HEADER_LEN: usize = 8;

// It looks like const_assert! doesn't count as being used somehow.
#[allow(dead_code)]
const EDID_IDENTIFICATION_LEN: usize = 10;
const EDID_MANUFACTURER_LEN: usize = 2;
const EDID_MANUFACTURER_CHAR_LEN: usize = 3;
const EDID_PRODUCT_CODE_LEN: usize = 2;
const EDID_SERIAL_NUMBER_LEN: usize = 4;
const EDID_DATE_LEN: usize = 2;

const_assert_eq!(
    EDID_MANUFACTURER_LEN + EDID_PRODUCT_CODE_LEN + EDID_SERIAL_NUMBER_LEN + EDID_DATE_LEN,
    EDID_IDENTIFICATION_LEN
);

// It looks like const_assert! doesn't count as being used somehow.
#[allow(dead_code)]
const EDID_VERSION_REVISION_LEN: usize = 2;

const EDID_BASIC_DISPLAY_PARAMETERS_LEN: usize = 5;
const EDID_INPUT_DEFINITION_LEN: usize = 1;
const EDID_ASPECT_RATIO_LEN: usize = 2;
const EDID_GAMMA_LEN: usize = 1;
const EDID_FEATURE_LEN: usize = 1;

const_assert_eq!(
    EDID_INPUT_DEFINITION_LEN + EDID_ASPECT_RATIO_LEN + EDID_GAMMA_LEN + EDID_FEATURE_LEN,
    EDID_BASIC_DISPLAY_PARAMETERS_LEN
);

const EDID_CHROMATICITY_COORDINATES_LEN: usize = 10;
const EDID_ESTABLISHED_TIMINGS_LEN: usize = 3;
const EDID_STANDARD_TIMINGS_LEN: usize = 16;
const EDID_DESCRIPTOR_LEN: usize = 18;
const EDID_DESCRIPTORS_NUM: usize = 4;
const EDID_DESCRIPTOR_PAYLOAD_LEN: usize = 13;

// It looks like const_assert! doesn't count as being used somehow.
#[allow(dead_code)]
const EDID_EXTENSION_NUM_LEN: usize = 1;

// It looks like const_assert! doesn't count as being used somehow.
#[allow(dead_code)]
const EDID_CHECKSUM_LEN: usize = 1;

const_assert_eq!(
    EDID_HEADER_LEN
        + EDID_IDENTIFICATION_LEN
        + EDID_VERSION_REVISION_LEN
        + EDID_BASIC_DISPLAY_PARAMETERS_LEN
        + EDID_CHROMATICITY_COORDINATES_LEN
        + EDID_ESTABLISHED_TIMINGS_LEN
        + EDID_STANDARD_TIMINGS_LEN
        + (EDID_DESCRIPTOR_LEN * EDID_DESCRIPTORS_NUM)
        + EDID_EXTENSION_NUM_LEN
        + EDID_CHECKSUM_LEN,
    EDID_BASE_LEN
);

pub trait IntoBytes {
    // Returns a serialized representation of the type. Must be of self.size() length.
    fn into_bytes(self) -> Vec<u8>;

    // Returns the byte length of the serialized representation of this type.
    fn size(&self) -> usize;
}

#[derive(Debug)]
pub enum EdidTypeConversionError<D: fmt::Display> {
    Int(num::TryFromIntError),
    Slice(array::TryFromSliceError),
    Range(D, Option<D>, Option<D>),
    Value(String),
}

impl<D: fmt::Display> From<num::TryFromIntError> for EdidTypeConversionError<D> {
    fn from(value: num::TryFromIntError) -> Self {
        EdidTypeConversionError::Int(value)
    }
}

impl<D: fmt::Display> From<array::TryFromSliceError> for EdidTypeConversionError<D> {
    fn from(value: array::TryFromSliceError) -> Self {
        EdidTypeConversionError::Slice(value)
    }
}

impl<D: fmt::Display> fmt::Display for EdidTypeConversionError<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdidTypeConversionError::Int(_) => write!(f, "Integer conversion error"),
            EdidTypeConversionError::Range(v, min, max) => {
                let range_str = if let (Some(min), Some(max)) = (min, max) {
                    format!("Range: {min}..={max}")
                } else if let Some(min) = min {
                    format!("Range: {min}..")
                } else if let Some(max) = max {
                    format!("Range: ..={max}")
                } else {
                    unimplemented!()
                };
                write!(f, "Value out of range: {v} ({range_str})")
            }
            EdidTypeConversionError::Slice(_) => write!(f, "Couldn't convert to an array"),
            EdidTypeConversionError::Value(s) => write!(f, "Invalid Value: {s}"),
        }
    }
}

impl<D: fmt::Display + fmt::Debug> core::error::Error for EdidTypeConversionError<D> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            EdidTypeConversionError::Int(e) => Some(e),
            EdidTypeConversionError::Slice(e) => Some(e),
            EdidTypeConversionError::Range(_, _, _) | EdidTypeConversionError::Value(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum EdidRelease {
    R3,
    R4,
}

#[derive(Clone, Copy, Debug)]
pub struct EdidManufacturer([u8; EDID_MANUFACTURER_CHAR_LEN]);

impl TryFrom<&str> for EdidManufacturer {
    type Error = EdidTypeConversionError<String>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !value.is_ascii() {
            return Err(EdidTypeConversionError::Value(String::from(
                "Manufacturer ID must be ASCII only.",
            )));
        }

        if !value.chars().all(char::is_uppercase) {
            return Err(EdidTypeConversionError::Value(String::from(
                "Manufacturer ID must be upper cased.",
            )));
        }

        if value.len() != EDID_MANUFACTURER_CHAR_LEN {
            return Err(EdidTypeConversionError::Value(String::from(
                "Manufacturer ID must be 3 characters long.",
            )));
        }

        Ok(Self(value.as_bytes().try_into()?))
    }
}

impl TryFrom<String> for EdidManufacturer {
    type Error = EdidTypeConversionError<String>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl IntoBytes for EdidManufacturer {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_MANUFACTURER_LEN);

        let manufacturer = &self.0;
        let mut comp = (manufacturer[0] - b'@') << 2;
        comp |= (manufacturer[1] - b'@') >> 3;
        bytes.push(comp);

        comp = (manufacturer[1] - b'@') << 5;
        comp |= manufacturer[2] - b'@';
        bytes.push(comp);

        let len = bytes.len();
        assert_eq!(
            len, EDID_MANUFACTURER_LEN,
            "Manufacturer array is larger than it should ({len} vs expected {EDID_MANUFACTURER_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_MANUFACTURER_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidProductCode(u16);

impl From<u16> for EdidProductCode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl IntoBytes for EdidProductCode {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_PRODUCT_CODE_LEN);

        let prod = &self.0;
        bytes.push((prod & 0xff) as u8);
        bytes.push((prod >> 8) as u8);

        let len = bytes.len();
        assert_eq!(
            len, EDID_PRODUCT_CODE_LEN,
            "Product Code array is larger than it should ({len} vs expected {EDID_PRODUCT_CODE_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_PRODUCT_CODE_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidSerialNumber(u32);

impl From<u32> for EdidSerialNumber {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl IntoBytes for EdidSerialNumber {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_SERIAL_NUMBER_LEN);

        let serial = &self.0;
        bytes.push((serial & 0xff) as u8);
        bytes.push(((serial >> 8) & 0xff) as u8);
        bytes.push(((serial >> 16) & 0xff) as u8);
        bytes.push(((serial >> 24) & 0xff) as u8);

        let len = bytes.len();
        assert_eq!(
            len, EDID_SERIAL_NUMBER_LEN,
            "Serial Number array is larger than it should ({len} vs expected {EDID_SERIAL_NUMBER_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_SERIAL_NUMBER_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidWeek(u8);

impl TryFrom<u8> for EdidWeek {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=53).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(53)));
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod test_edid_week {
    use crate::EdidWeek;

    #[test]
    fn test_range() {
        assert!(EdidWeek::try_from(0).is_err());
        assert!(EdidWeek::try_from(1).is_ok());
        assert!(EdidWeek::try_from(53).is_ok());
        assert!(EdidWeek::try_from(54).is_err());
    }
}

#[derive(Clone, Copy, Debug)]
struct EdidYear(u16);

impl TryFrom<u16> for EdidYear {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value < 1990 {
            return Err(EdidTypeConversionError::Range(value, Some(1990), None));
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod test_edid_year {
    use crate::EdidYear;

    #[test]
    fn test_range() {
        assert!(EdidYear::try_from(1989).is_err());
        assert!(EdidYear::try_from(1990).is_ok());
        assert!(EdidYear::try_from(2024).is_ok());
    }
}

/// EDID 1.3 Manufacture Date Representation.
///
/// Contains a year, starting from 1990, and an optional week in the 1-53 range.
#[derive(Clone, Copy, Debug)]
pub struct EdidManufactureDate(Option<EdidWeek>, EdidYear);

impl TryFrom<(u8, u16)> for EdidManufactureDate {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: (u8, u16)) -> Result<Self, Self::Error> {
        let week = value
            .0
            .try_into()
            .map_err(|e: EdidTypeConversionError<u8>| match e {
                EdidTypeConversionError::Int(e) => EdidTypeConversionError::Int(e),
                EdidTypeConversionError::Range(v, min, max) => {
                    EdidTypeConversionError::Range(v.into(), min.map(u16::from), max.map(u16::from))
                }
                EdidTypeConversionError::Slice(e) => EdidTypeConversionError::Slice(e),
                EdidTypeConversionError::Value(v) => EdidTypeConversionError::Value(v),
            })?;
        let year = value.1.try_into()?;

        Ok(Self(Some(week), year))
    }
}

impl TryFrom<u16> for EdidManufactureDate {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(None, value.try_into()?))
    }
}

impl IntoBytes for EdidManufactureDate {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DATE_LEN);

        let week = if let Some(val) = self.0 { val.0 } else { 0 };
        bytes.push(week);

        let year = u8::try_from(self.1 .0 - 1990).expect("Year would overflow our type.");
        bytes.push(year);

        let len = bytes.len();
        assert_eq!(
            len, EDID_DATE_LEN,
            "Date array is larger than it should ({len} vs expected {EDID_DATE_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DATE_LEN
    }
}

#[cfg(test)]
mod test_edid_manufacture_date {
    use crate::{EdidManufactureDate, IntoBytes};

    #[test]
    fn test_binary_spec() {
        // These are taken from the EDID 1.3 Specification, Section 3.4

        let date = EdidManufactureDate::try_from(1997).unwrap();
        assert_eq!(date.into_bytes(), &[0x00, 0x07]);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidR4Week(u8);

impl TryFrom<u8> for EdidR4Week {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=54).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(54)));
        }

        Ok(Self(value))
    }
}

#[cfg(test)]
mod test_edid_week_release_4 {
    use crate::EdidR4Week;

    #[test]
    fn test_range() {
        assert!(EdidR4Week::try_from(0).is_err());
        assert!(EdidR4Week::try_from(1).is_ok());
        assert!(EdidR4Week::try_from(54).is_ok());
        assert!(EdidR4Week::try_from(55).is_err());
    }
}

/// EDID 1.4 Manufacture Date Representation.
///
/// Contains a year, starting from 1990, and an optional week in the 1-54 range.
#[derive(Clone, Copy, Debug)]
pub struct EdidR4ManufactureDate(Option<EdidR4Week>, EdidYear);

impl TryFrom<(u8, u16)> for EdidR4ManufactureDate {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: (u8, u16)) -> Result<Self, Self::Error> {
        let week = value
            .0
            .try_into()
            .map_err(|e: EdidTypeConversionError<u8>| match e {
                EdidTypeConversionError::Int(e) => EdidTypeConversionError::Int(e),
                EdidTypeConversionError::Range(v, min, max) => {
                    EdidTypeConversionError::Range(v.into(), min.map(u16::from), max.map(u16::from))
                }
                EdidTypeConversionError::Slice(e) => EdidTypeConversionError::Slice(e),
                EdidTypeConversionError::Value(v) => EdidTypeConversionError::Value(v),
            })?;
        let year = value.1.try_into()?;

        Ok(Self(Some(week), year))
    }
}

impl TryFrom<u16> for EdidR4ManufactureDate {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(None, value.try_into()?))
    }
}

impl IntoBytes for EdidR4ManufactureDate {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DATE_LEN);

        let week = if let Some(val) = self.0 { val.0 } else { 0 };
        bytes.push(week);

        let year = u8::try_from(self.1 .0 - 1990).expect("Year would overflow our type.");
        bytes.push(year);

        let len = bytes.len();
        assert_eq!(
            len, EDID_DATE_LEN,
            "Date array is larger than it should ({len} vs expected {EDID_DATE_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DATE_LEN
    }
}

#[cfg(test)]
mod test_edid_manufacture_date_release_4 {
    use crate::{EdidManufactureDate, IntoBytes};

    #[test]
    fn test_binary_spec() {
        // This is taken from the EDID 1.4 Specification, Section 3.4.4, Example 1.

        let date = EdidManufactureDate::try_from((1, 2006)).unwrap();
        assert_eq!(date.into_bytes(), &[0x01, 0x10]);
    }
}

/// EDID 1.4 Model Date Representation.
///
/// Contains a Year, starting from 1990.
#[derive(Clone, Copy, Debug)]
pub struct EdidR4ModelDate(EdidYear);

impl TryFrom<u16> for EdidR4ModelDate {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(value.try_into()?))
    }
}

impl IntoBytes for EdidR4ModelDate {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_DATE_LEN);

        bytes.push(0xff);

        let year = u8::try_from(self.0 .0 - 1990).expect("Year would overflow our type.");
        bytes.push(year);

        let len = bytes.len();
        assert_eq!(
            len, EDID_DATE_LEN,
            "Date array is larger than it should ({len} vs expected {EDID_DATE_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_DATE_LEN
    }
}

#[cfg(test)]
mod test_edid_model_date {
    use crate::{EdidR4ModelDate, IntoBytes};

    #[test]
    fn test_binary_spec() {
        // This is taken from the EDID 1.4 Specification, Section 3.4.4, Example 2.

        let date: EdidR4ModelDate = 2006.try_into().unwrap();
        assert_eq!(date.into_bytes(), &[0xff, 0x10]);
    }
}

/// EDID 1.4 Date Representation.
#[derive(Clone, Copy, Debug)]
pub enum EdidR4Date {
    Manufacture(EdidR4ManufactureDate),
    Model(EdidR4ModelDate),
}

impl IntoBytes for EdidR4Date {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            EdidR4Date::Manufacture(m) => m.into_bytes(),
            EdidR4Date::Model(m) => m.into_bytes(),
        }
    }

    fn size(&self) -> usize {
        match self {
            EdidR4Date::Manufacture(m) => m.size(),
            EdidR4Date::Model(m) => m.size(),
        }
    }
}

#[cfg(test)]
mod test_edid_date_release_4 {
    use crate::{EdidR4Date, EdidR4ManufactureDate, EdidR4ModelDate, IntoBytes};

    #[test]
    fn test_binary_spec() {
        // These are taken from the EDID 1.4 Specification, Section 3.4.4

        let date = EdidR4Date::Manufacture(EdidR4ManufactureDate::try_from((1, 2006)).unwrap());
        assert_eq!(date.into_bytes(), &[0x01, 0x10]);

        let date = EdidR4Date::Model(EdidR4ModelDate::try_from(2006).unwrap());
        assert_eq!(date.into_bytes(), &[0xff, 0x10]);
    }
}

/// EDID Date Representation.
#[derive(Clone, Copy, Debug)]
pub enum EdidDate {
    R3(EdidManufactureDate),
    R4(EdidR4Date),
}

impl IntoBytes for EdidDate {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            EdidDate::R3(v) => v.into_bytes(),
            EdidDate::R4(v) => v.into_bytes(),
        };

        let len = bytes.len();
        let size = self.size();
        assert_eq!(
            len, size,
            "Date array is larger than it should ({len} vs expected {size} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        match self {
            EdidDate::R3(v) => v.size(),
            EdidDate::R4(v) => v.size(),
        }
    }
}

#[cfg(test)]
mod test_edid_date {
    use crate::{
        EdidDate, EdidManufactureDate, EdidR4Date, EdidR4ManufactureDate, EdidR4ModelDate,
        IntoBytes,
    };

    #[test]
    fn test_binary_spec() {
        // These are taken from the EDID 1.3 Specification, Section 3.4, and EDID 1.4 Specification,
        // Section 3.4.4.

        let date = EdidDate::R3(EdidManufactureDate::try_from(1997).unwrap());
        assert_eq!(date.into_bytes(), &[0x00, 0x07]);

        let date = EdidDate::R4(EdidR4Date::Manufacture(
            EdidR4ManufactureDate::try_from((1, 2006)).unwrap(),
        ));
        assert_eq!(date.into_bytes(), &[0x01, 0x10]);

        let date = EdidDate::R4(EdidR4Date::Model(EdidR4ModelDate::try_from(2006).unwrap()));
        assert_eq!(date.into_bytes(), &[0xff, 0x10]);
    }
}

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum EdidAnalogSignalLevelStandard {
    V_0_700_S_0_300_T_1_000 = 0,
    V_0_714_S_0_286_T_1_000,
    V_1_000_S_0_400_T_1_400,
    V_0_700_S_0_000_T_0_700,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EdidAnalogVideoSetup {
    BlankLevelIsBlackLevel = 0,
    BlankToBlackSetupOrPedestal,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidAnalogVideoInputDefinition {
    signal_level: EdidAnalogSignalLevelStandard,
    setup: EdidAnalogVideoSetup,

    #[builder(default)]
    separate_hv_sync_signals: bool,

    #[builder(default)]
    composite_sync_signal_on_hsync: bool,

    #[builder(default)]
    composite_sync_signal_on_green_video: bool,

    #[builder(default)]
    serrations_on_vsync: bool,
}

impl IntoBytes for EdidAnalogVideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        let mut byte = 0;

        byte |= (self.signal_level as u8) << 5;
        byte |= (self.setup as u8) << 4;

        if self.separate_hv_sync_signals {
            byte |= 1 << 3;
        }

        if self.composite_sync_signal_on_hsync {
            byte |= 1 << 2;
        }

        if self.composite_sync_signal_on_green_video {
            byte |= 1 << 1;
        }

        if self.serrations_on_vsync {
            byte |= 1 << 0;
        }

        let bytes = Vec::from(&[byte]);

        let len = bytes.len();
        assert_eq!(
            len, EDID_INPUT_DEFINITION_LEN,
            "Video Input Definition array is larger than it should ({len} vs expected {EDID_INPUT_DEFINITION_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_INPUT_DEFINITION_LEN
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidR3DigitalVideoInputDefinition {
    #[builder(default)]
    dfp1_compatible: bool,
}

impl IntoBytes for EdidR3DigitalVideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        let mut byte = 0x80;

        if self.dfp1_compatible {
            byte |= 1;
        }

        Vec::from(&[byte])
    }

    fn size(&self) -> usize {
        EDID_INPUT_DEFINITION_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR3VideoInputDefinition {
    Analog(EdidAnalogVideoInputDefinition),
    Digital(EdidR3DigitalVideoInputDefinition),
}

impl IntoBytes for EdidR3VideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            Self::Analog(v) => v.into_bytes(),
            Self::Digital(v) => v.into_bytes(),
        };

        let len = bytes.len();
        let size = self.size();
        assert_eq!(
            len, size,
            "Video Input Definition array is larger than it should ({len} vs expected {size} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        match self {
            EdidR3VideoInputDefinition::Analog(v) => v.size(),
            EdidR3VideoInputDefinition::Digital(v) => v.size(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidScreenSizeLength(u8);

impl TryFrom<u8> for EdidScreenSizeLength {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=255).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(255)));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidScreenSize {
    horizontal_cm: EdidScreenSizeLength,
    vertical_cm: EdidScreenSizeLength,
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR3ImageSize {
    Size(EdidScreenSize),
    Undefined,
}

impl IntoBytes for EdidR3ImageSize {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = Vec::from(&match self {
            Self::Size(s) => [s.horizontal_cm.0, s.vertical_cm.0],
            Self::Undefined => [0x00, 0x00],
        });

        let len = bytes.len();
        assert_eq!(
            len, EDID_ASPECT_RATIO_LEN,
            "Image Size array is larger than it should ({len} vs expected {EDID_ASPECT_RATIO_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_ASPECT_RATIO_LEN
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EdidDisplayColorType {
    MonochromeGrayScale = 0,
    RGBColor,
    NonRGBColor,
    Undefined,
}

/// Display Transfer Characteristics (aka Gamma)
#[derive(Clone, Copy, Debug)]
pub enum EdidDisplayTransferCharacteristics {
    Gamma(f32),
    DisplayInformationExtension(()),
}

impl TryFrom<f32> for EdidDisplayTransferCharacteristics {
    type Error = EdidTypeConversionError<f32>;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !(1.0..=3.54).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1.0), Some(3.54)));
        }

        Ok(Self::Gamma(value))
    }
}

impl IntoBytes for EdidDisplayTransferCharacteristics {
    fn into_bytes(self) -> Vec<u8> {
        let stored = match self {
            EdidDisplayTransferCharacteristics::Gamma(v) => {
                let raw = (v * 100.0) - 100.0;

                raw.round()
                    .to_u8()
                    .expect("Gamma binary representation would overflow.")
            }
            EdidDisplayTransferCharacteristics::DisplayInformationExtension(()) => 0xff,
        };

        let bytes = Vec::from(&[stored]);
        let len = bytes.len();
        assert_eq!(
            len, EDID_GAMMA_LEN,
            "Display Transfer Characteristics array is larger than it should ({len} vs expected {EDID_GAMMA_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_GAMMA_LEN
    }
}

#[cfg(test)]
mod test_display_transfer_characteristics {
    use super::{EdidDisplayTransferCharacteristics, IntoBytes};

    #[test]
    fn test_binary_spec() {
        // These are taken from the EDID 1.4 Specification, Section 3.6.2
        let gamma: EdidDisplayTransferCharacteristics = 2.2_f32.try_into().unwrap();
        assert_eq!(gamma.into_bytes(), &[0x78]);

        let ext = EdidDisplayTransferCharacteristics::DisplayInformationExtension(());
        assert_eq!(ext.into_bytes(), &[0xff]);
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidR3FeatureSupport {
    #[builder(default)]
    standby: bool,

    #[builder(default)]
    suspend: bool,

    #[builder(default)]
    active_off_is_very_low_power: bool,
    display_type: EdidDisplayColorType,

    #[builder(default)]
    srgb_default_color_space: bool,

    #[builder(default)]
    default_gtf_supported: bool,
}

impl IntoBytes for EdidR3FeatureSupport {
    fn into_bytes(self) -> Vec<u8> {
        // Preferred timing mode is required for EDID 1.3
        let mut byte = 1 << 1;

        if self.standby {
            byte |= 1 << 7;
        }

        if self.suspend {
            byte |= 1 << 6;
        }

        if self.active_off_is_very_low_power {
            byte |= 1 << 5;
        }

        byte |= (self.display_type as u8) << 3;

        if self.srgb_default_color_space {
            byte |= 1 << 2;
        }

        if self.default_gtf_supported {
            byte |= 1 << 0;
        }

        let bytes = Vec::from(&[byte]);
        let len = bytes.len();
        assert_eq!(
            len, EDID_FEATURE_LEN,
            "Basic Features array is larger than it should ({len} vs expected {EDID_FEATURE_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_FEATURE_LEN
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidR3BasicDisplayParametersFeatures {
    video_input: EdidR3VideoInputDefinition,
    size: EdidR3ImageSize,

    #[builder(setter(into))]
    display_transfer_characteristic: EdidDisplayTransferCharacteristics,

    feature_support: EdidR3FeatureSupport,
}

impl IntoBytes for EdidR3BasicDisplayParametersFeatures {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_BASIC_DISPLAY_PARAMETERS_LEN);

        bytes.extend_from_slice(&self.video_input.into_bytes());
        bytes.extend_from_slice(&self.size.into_bytes());
        bytes.extend_from_slice(&self.display_transfer_characteristic.into_bytes());
        bytes.extend_from_slice(&self.feature_support.into_bytes());

        let len = bytes.len();
        assert_eq!(
            len, EDID_BASIC_DISPLAY_PARAMETERS_LEN,
            "Basic Features array is larger than it should ({len} vs expected {EDID_BASIC_DISPLAY_PARAMETERS_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_BASIC_DISPLAY_PARAMETERS_LEN
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EdidR4DigitalColorDepth {
    DepthUndefined = 0,
    Depth6Bpc,
    Depth8Bpc,
    Depth10Bpc,
    Depth12Bpc,
    Depth14Bpc,
    Depth16Bpc,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EdidR4DigitalInterface {
    Undefined = 0,
    DVI,
    HDMIa,
    HDMIb,
    MDDI,
    DisplayPort,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidR4DigitalVideoInputDefinition {
    color_depth: EdidR4DigitalColorDepth,
    interface: EdidR4DigitalInterface,
}

impl IntoBytes for EdidR4DigitalVideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        let mut byte: u8 = 1 << 7;

        byte |= (self.color_depth as u8) << 4;
        byte |= self.interface as u8;

        let bytes = Vec::from(&[byte]);
        let len = bytes.len();
        assert_eq!(
            len, EDID_INPUT_DEFINITION_LEN,
            "Video Input Definition array is larger than it should ({len} vs expected {EDID_INPUT_DEFINITION_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_INPUT_DEFINITION_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR4VideoInputDefinition {
    Analog(EdidAnalogVideoInputDefinition),
    Digital(EdidR4DigitalVideoInputDefinition),
}

impl IntoBytes for EdidR4VideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            Self::Analog(v) => v.into_bytes(),
            Self::Digital(v) => v.into_bytes(),
        };

        let len = bytes.len();
        let size = self.size();
        assert_eq!(
            len, size,
            "Video Input Definition array is larger than it should ({len} vs expected {size} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        match self {
            EdidR4VideoInputDefinition::Analog(v) => v.size(),
            EdidR4VideoInputDefinition::Digital(v) => v.size(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidR4ImageLandscapeAspectRatio(f32, f32);

impl TryFrom<(f32, f32)> for EdidR4ImageLandscapeAspectRatio {
    type Error = EdidTypeConversionError<f32>;

    fn try_from(value: (f32, f32)) -> Result<Self, Self::Error> {
        let num = value.0;
        let denum = value.1;

        let num = num / denum;
        if !(1.0..=3.54).contains(&num) {
            return Err(EdidTypeConversionError::Range(num, Some(1.0), Some(3.54)));
        }

        Ok(Self(num, 1.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidR4ImagePortraitAspectRatio(f32, f32);

impl TryFrom<(f32, f32)> for EdidR4ImagePortraitAspectRatio {
    type Error = EdidTypeConversionError<f32>;

    fn try_from(value: (f32, f32)) -> Result<Self, Self::Error> {
        let num = value.0;
        let denum = value.1;

        let num = num / denum;
        if !(0.28..=0.99).contains(&num) {
            return Err(EdidTypeConversionError::Range(num, Some(0.28), Some(0.99)));
        }

        Ok(Self(num, 1.0))
    }
}

/// EDID 1.4 Screen Size or Aspect Ratio
///
/// For displays that pivot, the screen size is considered in landscape mode.
#[derive(Clone, Copy, Debug)]
pub enum EdidR4ImageSize {
    LandscapeRatio(EdidR4ImageLandscapeAspectRatio),
    PortraitRatio(EdidR4ImagePortraitAspectRatio),
    Size(EdidScreenSize),
    Undefined,
}

impl IntoBytes for EdidR4ImageSize {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = Vec::from(&match self {
            Self::LandscapeRatio(r) => {
                let ratio = f64::from(r.0) / f64::from(r.1);
                let ratio_cent_int = (ratio * 100.0)
                    .round()
                    .to_u16()
                    .expect("Couldn't convert our aspect ratio to an integer.");
                let stored = u8::try_from(ratio_cent_int - 99)
                    .expect("Aspect Ratio would overflow our type.");

                [stored, 0x00]
            }
            Self::PortraitRatio(r) => {
                let ratio = f64::from(r.0) / f64::from(r.1);
                let ratio_cent_inv = (100.0 / ratio)
                    .round()
                    .to_u16()
                    .expect("Couldn't convert our aspect ratio to an integer.");
                let stored = u8::try_from(ratio_cent_inv - 99)
                    .expect("Aspect Ratio would overflow our type.");

                [0x00, stored]
            }
            Self::Size(s) => [s.horizontal_cm.0, s.vertical_cm.0],
            Self::Undefined => [0x00, 0x00],
        });

        let len = bytes.len();
        assert_eq!(
            len, EDID_ASPECT_RATIO_LEN,
            "Image Size array is larger than it should ({len} vs expected {EDID_ASPECT_RATIO_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_ASPECT_RATIO_LEN
    }
}

#[cfg(test)]
mod test_size_release_4 {
    use super::{
        EdidR4ImageLandscapeAspectRatio, EdidR4ImagePortraitAspectRatio, EdidR4ImageSize, IntoBytes,
    };

    #[test]
    fn test_binary_spec() {
        // These are taken from the EDID 1.4 Specification, Section 3.6.2
        let undef = EdidR4ImageSize::Undefined;
        assert_eq!(undef.into_bytes(), &[0x00, 0x00]);

        let ratio: EdidR4ImageLandscapeAspectRatio = (16.0, 9.0).try_into().unwrap();
        let landscape = EdidR4ImageSize::LandscapeRatio(ratio);
        assert_eq!(landscape.into_bytes(), &[0x4f, 0x00]);

        let ratio: EdidR4ImageLandscapeAspectRatio = (16.0, 10.0).try_into().unwrap();
        let landscape = EdidR4ImageSize::LandscapeRatio(ratio);
        assert_eq!(landscape.into_bytes(), &[0x3d, 0x00]);

        let ratio: EdidR4ImageLandscapeAspectRatio = (4.0, 3.0).try_into().unwrap();
        let landscape = EdidR4ImageSize::LandscapeRatio(ratio);
        assert_eq!(landscape.into_bytes(), &[0x22, 0x00]);

        let ratio: EdidR4ImageLandscapeAspectRatio = (5.0, 4.0).try_into().unwrap();
        let landscape = EdidR4ImageSize::LandscapeRatio(ratio);
        assert_eq!(landscape.into_bytes(), &[0x1a, 0x00]);

        let ratio: EdidR4ImagePortraitAspectRatio = (9.0, 16.0).try_into().unwrap();
        let portrait = EdidR4ImageSize::PortraitRatio(ratio);
        assert_eq!(portrait.into_bytes(), &[0x00, 0x4f]);

        let ratio: EdidR4ImagePortraitAspectRatio = (10.0, 16.0).try_into().unwrap();
        let portrait = EdidR4ImageSize::PortraitRatio(ratio);
        assert_eq!(portrait.into_bytes(), &[0x00, 0x3d]);

        let ratio: EdidR4ImagePortraitAspectRatio = (3.0, 4.0).try_into().unwrap();
        let portrait = EdidR4ImageSize::PortraitRatio(ratio);
        assert_eq!(portrait.into_bytes(), &[0x00, 0x22]);

        let ratio: EdidR4ImagePortraitAspectRatio = (4.0, 5.0).try_into().unwrap();
        let portrait = EdidR4ImageSize::PortraitRatio(ratio);
        assert_eq!(portrait.into_bytes(), &[0x00, 0x1a]);
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EdidR4DisplayColorEncoding {
    RGB444 = 0,
    RGB444YCbCr444,
    RGB444YCbCr422,
    RGB444YCbCr444YCbCr422,
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR4DisplayColor {
    Analog(EdidDisplayColorType),
    Digital(EdidR4DisplayColorEncoding),
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidR4FeatureSupport {
    #[builder(default)]
    #[deprecated]
    standby: bool,

    #[builder(default)]
    #[deprecated]
    suspend: bool,

    #[builder(default)]
    active_off_is_very_low_power: bool,
    color: EdidR4DisplayColor,

    #[builder(default)]
    srgb_default_color_space: bool,

    #[builder(default)]
    preferred_timing_mode_is_native: bool,

    #[builder(default)]
    continuous_frequency: bool,
}

impl IntoBytes for EdidR4FeatureSupport {
    fn into_bytes(self) -> Vec<u8> {
        let mut byte = 0;

        #[allow(deprecated)]
        if self.standby {
            byte |= 1 << 7;
        }

        #[allow(deprecated)]
        if self.suspend {
            byte |= 1 << 6;
        }

        if self.active_off_is_very_low_power {
            byte |= 1 << 5;
        }

        // FIXME: This should be cross-checked with the content of EdidR4VideoInputDefinition.
        byte |= match self.color {
            EdidR4DisplayColor::Analog(a) => a as u8,
            EdidR4DisplayColor::Digital(d) => d as u8,
        } << 3;

        if self.srgb_default_color_space {
            byte |= 1 << 2;
        }

        if self.preferred_timing_mode_is_native {
            byte |= 1 << 1;
        }

        if self.continuous_frequency {
            byte |= 1 << 0;
        }

        let bytes = Vec::from(&[byte]);
        let len = bytes.len();
        assert_eq!(
            len, EDID_FEATURE_LEN,
            "Feature array is larger than it should ({len} vs expected {EDID_FEATURE_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_FEATURE_LEN
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidR4BasicDisplayParametersFeatures {
    video_input: EdidR4VideoInputDefinition,
    size: EdidR4ImageSize,

    #[builder(setter(into))]
    display_transfer_characteristic: EdidDisplayTransferCharacteristics,
    feature_support: EdidR4FeatureSupport,
}

impl IntoBytes for EdidR4BasicDisplayParametersFeatures {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_BASIC_DISPLAY_PARAMETERS_LEN);

        bytes.extend_from_slice(&self.video_input.into_bytes());
        bytes.extend_from_slice(&self.size.into_bytes());
        bytes.extend_from_slice(&self.display_transfer_characteristic.into_bytes());
        bytes.extend_from_slice(&self.feature_support.into_bytes());

        let len = bytes.len();
        assert_eq!(
            len, EDID_BASIC_DISPLAY_PARAMETERS_LEN,
            "Basic Display Parameters array is larger than it should ({len} vs expected {EDID_BASIC_DISPLAY_PARAMETERS_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_BASIC_DISPLAY_PARAMETERS_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidBasicDisplayParametersFeatures {
    R3(EdidR3BasicDisplayParametersFeatures),
    R4(EdidR4BasicDisplayParametersFeatures),
}

impl IntoBytes for EdidBasicDisplayParametersFeatures {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            Self::R3(v) => v.into_bytes(),
            Self::R4(v) => v.into_bytes(),
        };

        let len = bytes.len();
        let size = self.size();
        assert_eq!(
            len, size,
            "Basic Display Parameters array is larger than it should ({len} vs expected {size} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        match self {
            EdidBasicDisplayParametersFeatures::R3(v) => v.size(),
            EdidBasicDisplayParametersFeatures::R4(v) => v.size(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidChromaticityCoordinate(f32);

impl EdidChromaticityCoordinate {
    fn into_raw(self) -> u16 {
        (self.0 * 1024.0)
            .round()
            .to_u16()
            .expect("Chromaticity Coordinate binary representation would overflow.")
    }
}

#[cfg(test)]
mod test_chromaticity_coordinate {
    use super::EdidChromaticityCoordinate;

    #[test]
    fn test_binary_spec() {
        assert_eq!(EdidChromaticityCoordinate(0.610).into_raw(), 0b10_0111_0001);
        assert_eq!(EdidChromaticityCoordinate(0.307).into_raw(), 0b01_0011_1010);
        assert_eq!(EdidChromaticityCoordinate(0.150).into_raw(), 0b00_1001_1010);
    }
}

impl TryFrom<f32> for EdidChromaticityCoordinate {
    type Error = EdidTypeConversionError<f32>;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !(0.0..=1.0).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(0.0), Some(1.0)));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidChromaticityPoint(EdidChromaticityCoordinate, EdidChromaticityCoordinate);

impl TryFrom<(f32, f32)> for EdidChromaticityPoint {
    type Error = EdidTypeConversionError<f32>;

    fn try_from(value: (f32, f32)) -> Result<Self, Self::Error> {
        let x = value.0.try_into()?;
        let y = value.1.try_into()?;

        Ok(Self(x, y))
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct EdidChromaticityPoints {
    white: EdidChromaticityPoint,
    red: EdidChromaticityPoint,
    green: EdidChromaticityPoint,
    blue: EdidChromaticityPoint,
}

#[allow(variant_size_differences)]
#[derive(Clone, Copy, Debug)]
pub enum EdidFilterChromaticity {
    // FIXME: This must be consistent with EdidDisplayColorType.
    MonoChrome(EdidChromaticityPoint),
    Color(EdidChromaticityPoints),
}

impl IntoBytes for EdidFilterChromaticity {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            EdidFilterChromaticity::MonoChrome(white) => {
                let white_x = white.0.into_raw();
                let white_x_lo = (white_x & 0b11) as u8;
                let white_x_hi = ((white_x >> 2) & 0xff) as u8;
                let white_y = white.1.into_raw();
                let white_y_lo = (white_y & 0b11) as u8;
                let white_y_hi = ((white_y >> 2) & 0xff) as u8;

                [
                    0x00,
                    white_x_lo << 2 | white_y_lo,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    white_x_hi,
                    white_y_hi,
                ]
            }
            EdidFilterChromaticity::Color(points) => {
                let white = &points.white;
                let white_x = white.0.into_raw();
                let white_x_lo = (white_x & 0b11) as u8;
                let white_x_hi = ((white_x >> 2) & 0xff) as u8;
                let white_y = white.1.into_raw();
                let white_y_lo = (white_y & 0b11) as u8;
                let white_y_hi = ((white_y >> 2) & 0xff) as u8;
                let red = &points.red;
                let red_x = red.0.into_raw();
                let red_x_lo = (red_x & 0b11) as u8;
                let red_x_hi = ((red_x >> 2) & 0xff) as u8;
                let red_y = red.1.into_raw();
                let red_y_lo = (red_y & 0b11) as u8;
                let red_y_hi = ((red_y >> 2) & 0xff) as u8;
                let green = &points.green;
                let green_x = green.0.into_raw();
                let green_x_lo = (green_x & 0b11) as u8;
                let green_x_hi = ((green_x >> 2) & 0xff) as u8;
                let green_y = green.1.into_raw();
                let green_y_lo = (green_y & 0b11) as u8;
                let green_y_hi = ((green_y >> 2) & 0xff) as u8;
                let blue = &points.blue;
                let blue_x = blue.0.into_raw();
                let blue_x_lo = (blue_x & 0b11) as u8;
                let blue_x_hi = ((blue_x >> 2) & 0xff) as u8;
                let blue_y = blue.1.into_raw();
                let blue_y_lo = (blue_y & 0b11) as u8;
                let blue_y_hi = ((blue_y >> 2) & 0xff) as u8;

                [
                    red_x_lo << 6 | red_y_lo << 4 | green_x_lo << 2 | green_y_lo,
                    blue_x_lo << 6 | blue_y_lo << 4 | white_x_lo << 2 | white_y_lo,
                    red_x_hi,
                    red_y_hi,
                    green_x_hi,
                    green_y_hi,
                    blue_x_hi,
                    blue_y_hi,
                    white_x_hi,
                    white_y_hi,
                ]
            }
        };

        let bytes = Vec::from(&bytes);
        let len = bytes.len();
        assert_eq!(
            len, EDID_CHROMATICITY_COORDINATES_LEN,
            "Basic Display Parameters array is larger than it should ({len} vs expected {EDID_CHROMATICITY_COORDINATES_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_CHROMATICITY_COORDINATES_LEN
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum EdidEstablishedTiming {
    ET_1024_768_60hz,
    ET_1024_768_70hz,
    ET_1024_768_75hz,
    ET_1024_768_87hz_Interlaced,
    ET_1152_870_75hz,
    ET_1280_1024_75hz,
    ET_640_480_60hz,
    ET_640_480_67hz,
    ET_640_480_72hz,
    ET_640_480_75hz,
    ET_720_400_70hz,
    ET_720_400_88hz,
    ET_800_600_56hz,
    ET_800_600_60hz,
    ET_800_600_72hz,
    ET_800_600_75hz,
    ET_832_624_75hz,
    Manufacturer0,
    Manufacturer1,
    Manufacturer2,
    Manufacturer3,
    Manufacturer4,
    Manufacturer5,
    Manufacturer6,
}

impl IntoBytes for Vec<EdidEstablishedTiming> {
    fn into_bytes(self) -> Vec<u8> {
        let mut byte0: u8 = 0;
        let mut byte1: u8 = 0;
        let mut byte2: u8 = 0;
        for et in self {
            match et {
                EdidEstablishedTiming::ET_800_600_60hz => byte0 |= 1 << 0,
                EdidEstablishedTiming::ET_800_600_56hz => byte0 |= 1 << 1,
                EdidEstablishedTiming::ET_640_480_75hz => byte0 |= 1 << 2,
                EdidEstablishedTiming::ET_640_480_72hz => byte0 |= 1 << 3,
                EdidEstablishedTiming::ET_640_480_67hz => byte0 |= 1 << 4,
                EdidEstablishedTiming::ET_640_480_60hz => byte0 |= 1 << 5,
                EdidEstablishedTiming::ET_720_400_88hz => byte0 |= 1 << 6,
                EdidEstablishedTiming::ET_720_400_70hz => byte0 |= 1 << 7,
                EdidEstablishedTiming::ET_1280_1024_75hz => byte1 |= 1 << 0,
                EdidEstablishedTiming::ET_1024_768_75hz => byte1 |= 1 << 1,
                EdidEstablishedTiming::ET_1024_768_70hz => byte1 |= 1 << 2,
                EdidEstablishedTiming::ET_1024_768_60hz => byte1 |= 1 << 3,
                EdidEstablishedTiming::ET_1024_768_87hz_Interlaced => byte1 |= 1 << 4,
                EdidEstablishedTiming::ET_832_624_75hz => byte1 |= 1 << 5,
                EdidEstablishedTiming::ET_800_600_75hz => byte1 |= 1 << 6,
                EdidEstablishedTiming::ET_800_600_72hz => byte1 |= 1 << 7,
                EdidEstablishedTiming::ET_1152_870_75hz => byte2 |= 1 << 7,
                EdidEstablishedTiming::Manufacturer0 => byte2 |= 1 << 0,
                EdidEstablishedTiming::Manufacturer1 => byte2 |= 1 << 1,
                EdidEstablishedTiming::Manufacturer2 => byte2 |= 1 << 2,
                EdidEstablishedTiming::Manufacturer3 => byte2 |= 1 << 3,
                EdidEstablishedTiming::Manufacturer4 => byte2 |= 1 << 4,
                EdidEstablishedTiming::Manufacturer5 => byte2 |= 1 << 5,
                EdidEstablishedTiming::Manufacturer6 => byte2 |= 1 << 6,
            }
        }

        let bytes = Vec::from(&[byte0, byte1, byte2]);
        let len = bytes.len();
        assert_eq!(
            len, EDID_ESTABLISHED_TIMINGS_LEN,
            "Established Timings array is larger than it should ({len} vs expected {EDID_ESTABLISHED_TIMINGS_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_ESTABLISHED_TIMINGS_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidStandardTimingHorizontalSize(u16);

impl TryFrom<u16> for EdidStandardTimingHorizontalSize {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(256..=2288).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(256), Some(2288)));
        }

        if (value % 8) != 0 {
            return Err(EdidTypeConversionError::Value(String::from(
                "Standard Timing Horizontal Size must be a multiple of 8 pixels.",
            )));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidStandardTimingRefreshRate(u8);

impl TryFrom<u8> for EdidStandardTimingRefreshRate {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(60..=123).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(60), Some(123)));
        }

        Ok(Self(value))
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum EdidStandardTimingRatio {
    Ratio_16_10,
    Ratio_4_3,
    Ratio_5_4,
    Ratio_16_9,
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct EdidStandardTiming {
    x: EdidStandardTimingHorizontalSize,
    ratio: EdidStandardTimingRatio,
    frequency: EdidStandardTimingRefreshRate,
}

impl IntoBytes for Vec<EdidStandardTiming> {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(EDID_STANDARD_TIMINGS_LEN);

        for st_idx in 0..8 {
            let st = self.get(st_idx);
            match st {
                Some(timing) => {
                    let byte0 = u8::try_from((timing.x.0 / 8) - 31)
                        .expect("Standard Timing X Value is too big");

                    let mut byte1 = (timing.frequency.0 - 60) & 0x3f;
                    let ratio: u8 = match timing.ratio {
                        EdidStandardTimingRatio::Ratio_16_10 => 0,
                        EdidStandardTimingRatio::Ratio_4_3 => 1,
                        EdidStandardTimingRatio::Ratio_5_4 => 2,
                        EdidStandardTimingRatio::Ratio_16_9 => 3,
                    };
                    byte1 |= ratio << 6;

                    bytes.extend_from_slice(&[byte0, byte1]);
                }
                None => bytes.extend_from_slice(&[0x01, 0x01]),
            }
        }

        let len = bytes.len();
        assert_eq!(
            len, EDID_STANDARD_TIMINGS_LEN,
            "Standard timings array is larger than it should ({len} vs expected {EDID_STANDARD_TIMINGS_LEN} bytes)",
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_STANDARD_TIMINGS_LEN
    }
}

#[cfg(test)]
mod test_edid_standard_timings {
    use crate::{EdidStandardTiming, EdidStandardTimingRatio, IntoBytes};

    #[test]
    fn test_binary_spec() {
        let timings = vec![EdidStandardTiming {
            x: 1920.try_into().unwrap(),
            ratio: EdidStandardTimingRatio::Ratio_16_9,
            frequency: 60.try_into().unwrap(),
        }];
        assert_eq!(
            timings.into_bytes(),
            &[
                0xd1, 0xc0, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
                0x01, 0x01,
            ]
        );
    }
}

#[derive(Clone, Debug)]
struct Edid {
    release: EdidRelease,
    manufacturer: EdidManufacturer,
    product_code: EdidProductCode,
    serial_number: Option<EdidSerialNumber>,
    date: EdidDate,
    bdpf: EdidBasicDisplayParametersFeatures,
    chroma_coord: EdidFilterChromaticity,
    established_timings: Vec<EdidEstablishedTiming>,
    standard_timings: Vec<EdidStandardTiming>,
    descriptors: Vec<EdidDescriptor>,
    extensions: Vec<EdidExtension>,
}

impl IntoBytes for Edid {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(0x80);

        bytes.extend_from_slice(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]);

        bytes.extend_from_slice(&self.manufacturer.into_bytes());
        bytes.extend_from_slice(&self.product_code.into_bytes());

        if let Some(sn) = self.serial_number {
            bytes.extend_from_slice(&sn.into_bytes());
        } else {
            bytes.extend_from_slice(&[0x00; 4]);
        }

        bytes.extend_from_slice(&self.date.into_bytes());

        bytes.extend_from_slice(match self.release {
            EdidRelease::R3 => &[1, 3],
            EdidRelease::R4 => &[1, 4],
        });

        bytes.extend_from_slice(&self.bdpf.into_bytes());
        bytes.extend_from_slice(&self.chroma_coord.into_bytes());

        bytes.extend_from_slice(&self.established_timings.into_bytes());
        bytes.extend_from_slice(&self.standard_timings.into_bytes());
        bytes.extend_from_slice(&self.descriptors.into_bytes());

        let num_exts = self
            .extensions
            .len()
            .to_u8()
            .expect("Number of extensions would overflow our type.");
        bytes.push(num_exts);

        let mut sum: u8 = 0;
        for byte in &bytes {
            sum = sum.wrapping_add(*byte);
        }

        let checksum = 0u8.wrapping_sub(sum);
        bytes.push(checksum);

        for ext in self.extensions {
            bytes.extend_from_slice(&ext.into_bytes());
        }

        assert_eq!(
            bytes.len() % EDID_BASE_LEN,
            0,
            "EDID must be {EDID_BASE_LEN} bytes aligned (actual size {})",
            bytes.len()
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_BASE_LEN
    }
}

impl From<EdidRelease3> for Edid {
    fn from(value: EdidRelease3) -> Self {
        Self {
            release: EdidRelease::R3,
            manufacturer: value.manufacturer,
            product_code: value.product_code,
            serial_number: value.serial_number,
            date: EdidDate::R3(value.date),
            bdpf: EdidBasicDisplayParametersFeatures::R3(value.display_parameters_features),
            chroma_coord: value.filter_chromaticity,
            established_timings: value.established_timings,
            standard_timings: value.standard_timings,
            descriptors: value.descriptors,
            extensions: value.extensions,
        }
    }
}

impl From<EdidRelease4> for Edid {
    fn from(value: EdidRelease4) -> Self {
        Self {
            release: EdidRelease::R4,
            manufacturer: value.manufacturer,
            product_code: value.product_code,
            serial_number: value.serial_number,
            date: EdidDate::R4(value.date),
            bdpf: EdidBasicDisplayParametersFeatures::R4(value.display_parameters_features),
            chroma_coord: value.filter_chromaticity,
            established_timings: value.established_timings,
            standard_timings: value.standard_timings,
            descriptors: value.descriptors,
            extensions: value.extensions,
        }
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn descriptors(&mut self, d: Vec<EdidR3Descriptor>) {
        self.descriptors = d.into_iter().map(EdidDescriptor::R3).collect();
    }

    #[allow(unreachable_pub)]
    pub fn add_descriptor(&mut self, d: EdidR3Descriptor) {
        self.descriptors.push(EdidDescriptor::R3(d));
    }

    #[allow(unreachable_pub)]
    pub fn established_timings(&mut self, et: Vec<EdidEstablishedTiming>) {
        self.established_timings = et;
    }

    #[allow(unreachable_pub)]
    pub fn add_established_timing(&mut self, et: EdidEstablishedTiming) {
        self.established_timings.push(et);
    }

    #[allow(unreachable_pub)]
    pub fn standard_timings(&mut self, st: Vec<EdidStandardTiming>) {
        self.standard_timings = st;
    }

    #[allow(unreachable_pub)]
    pub fn add_standard_timing(&mut self, st: EdidStandardTiming) {
        self.standard_timings.push(st);
    }

    #[allow(unreachable_pub)]
    pub fn extensions(&mut self, ext: Vec<EdidExtension>) {
        self.extensions = ext;
    }

    #[allow(unreachable_pub)]
    pub fn add_extension(&mut self, ext: EdidExtension) {
        self.extensions.push(ext);
    }
))]
pub struct EdidRelease3 {
    manufacturer: EdidManufacturer,

    #[builder(setter(into))]
    product_code: EdidProductCode,

    #[builder(default)]
    serial_number: Option<EdidSerialNumber>,

    date: EdidManufactureDate,
    display_parameters_features: EdidR3BasicDisplayParametersFeatures,
    filter_chromaticity: EdidFilterChromaticity,

    #[builder(via_mutators, default = vec![EdidEstablishedTiming::ET_640_480_60hz])]
    established_timings: Vec<EdidEstablishedTiming>,

    #[builder(via_mutators)]
    standard_timings: Vec<EdidStandardTiming>,

    // FIXME: The Preferred Timing Descriptors is required in the first position
    // FIXME: Monitor Name is mandatory
    // FIXME: Display Range Limits is mandatory
    #[builder(via_mutators)]
    descriptors: Vec<EdidDescriptor>,

    #[builder(via_mutators)]
    extensions: Vec<EdidExtension>,
}

impl IntoBytes for EdidRelease3 {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = Edid::from(self).into_bytes();

        let len = bytes.len();
        assert_eq!(
            len % EDID_BASE_LEN,
            0,
            "EDID must be {EDID_BASE_LEN} bytes aligned (actual size {len})"
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_BASE_LEN
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn descriptors(&mut self, d: Vec<EdidR4Descriptor>) {
        self.descriptors = d.into_iter().map(EdidDescriptor::R4).collect();
    }

    #[allow(unreachable_pub)]
    pub fn add_descriptor(&mut self, d: EdidR4Descriptor) {
        self.descriptors.push(EdidDescriptor::R4(d));
    }

    #[allow(unreachable_pub)]
    pub fn established_timings(&mut self, et: Vec<EdidEstablishedTiming>) {
        self.established_timings = et;
    }

    #[allow(unreachable_pub)]
    pub fn add_established_timing(&mut self, et: EdidEstablishedTiming) {
        self.established_timings.push(et);
    }

    #[allow(unreachable_pub)]
    pub fn standard_timings(&mut self, st: Vec<EdidStandardTiming>) {
        self.standard_timings = st;
    }

    #[allow(unreachable_pub)]
    pub fn add_standard_timing(&mut self, st: EdidStandardTiming) {
        self.standard_timings.push(st);
    }

    #[allow(unreachable_pub)]
    pub fn extensions(&mut self, ext: Vec<EdidExtension>) {
        self.extensions = ext;
    }

    #[allow(unreachable_pub)]
    pub fn add_extension(&mut self, ext: EdidExtension) {
        self.extensions.push(ext);
    }
))]
pub struct EdidRelease4 {
    manufacturer: EdidManufacturer,

    #[builder(setter(into))]
    product_code: EdidProductCode,

    #[builder(default)]
    serial_number: Option<EdidSerialNumber>,

    date: EdidR4Date,
    display_parameters_features: EdidR4BasicDisplayParametersFeatures,
    filter_chromaticity: EdidFilterChromaticity,

    #[builder(via_mutators, default = vec![EdidEstablishedTiming::ET_640_480_60hz])]
    established_timings: Vec<EdidEstablishedTiming>,

    #[builder(via_mutators)]
    standard_timings: Vec<EdidStandardTiming>,

    // FIXME: The Preferred Timing Descriptors is required in the first position
    // FIXME: If continuous frequency, a display range limits descriptor is required
    #[builder(via_mutators)]
    descriptors: Vec<EdidDescriptor>,

    #[builder(via_mutators)]
    extensions: Vec<EdidExtension>,
}

impl IntoBytes for EdidRelease4 {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = Edid::from(self).into_bytes();

        let len = bytes.len();
        assert_eq!(
            len % EDID_BASE_LEN,
            0,
            "EDID must be {EDID_BASE_LEN} bytes aligned (actual size {len})"
        );

        bytes
    }

    fn size(&self) -> usize {
        EDID_BASE_LEN
    }
}

#[cfg(test)]
mod test_edid_release4 {
    use crate::{
        descriptors::EdidDetailedTimingPixelClock, EdidAnalogSignalLevelStandard,
        EdidAnalogVideoInputDefinition, EdidAnalogVideoSetup, EdidChromaticityPoint,
        EdidChromaticityPoints, EdidDescriptor10BitsTiming, EdidDescriptor12BitsTiming,
        EdidDescriptor6BitsTiming, EdidDescriptor8BitsTiming, EdidDescriptorDetailedTiming,
        EdidDescriptorString, EdidDetailedTimingDigitalSeparateSync, EdidDetailedTimingDigitalSync,
        EdidDetailedTimingDigitalSyncKind, EdidDetailedTimingSizeMm, EdidDetailedTimingStereo,
        EdidDetailedTimingSync, EdidDisplayColorType, EdidDisplayRangePixelClock,
        EdidDisplayRangeVerticalFreq, EdidDisplayTransferCharacteristics, EdidEstablishedTiming,
        EdidFilterChromaticity, EdidManufacturer, EdidProductCode,
        EdidR4BasicDisplayParametersFeatures, EdidR4Date, EdidR4Descriptor,
        EdidR4DescriptorEstablishedTimings, EdidR4DescriptorEstablishedTimingsIII,
        EdidR4DisplayColor, EdidR4DisplayRangeHorizontalFreq, EdidR4DisplayRangeLimits,
        EdidR4DisplayRangeVerticalFreq, EdidR4DisplayRangeVideoTimingsAspectRatio,
        EdidR4DisplayRangeVideoTimingsCVT, EdidR4DisplayRangeVideoTimingsCVTR1,
        EdidR4DisplayRangeVideoTimingsSupport, EdidR4FeatureSupport, EdidR4ImageSize,
        EdidR4ManufactureDate, EdidR4VideoInputDefinition, EdidRelease4, EdidScreenSize,
        EdidScreenSizeLength, EdidSerialNumber, EdidStandardTiming,
        EdidStandardTimingHorizontalSize, EdidStandardTimingRatio, EdidStandardTimingRefreshRate,
        IntoBytes,
    };

    #[test]
    fn test_binary_spec_example_1() {
        // This is taken from the EDID 1.4 Section 6.1

        let edid = EdidRelease4::builder()
            .manufacturer(EdidManufacturer::try_from("ABC").unwrap())
            .product_code(EdidProductCode::from(0xf206))
            .serial_number(Some(EdidSerialNumber::from(0x00000001)))
            .date(EdidR4Date::Manufacture(
                EdidR4ManufactureDate::try_from((1, 2007)).unwrap(),
            ))
            .display_parameters_features(
                EdidR4BasicDisplayParametersFeatures::builder()
                    .video_input(EdidR4VideoInputDefinition::Analog(
                        EdidAnalogVideoInputDefinition::builder()
                            .signal_level(EdidAnalogSignalLevelStandard::V_0_700_S_0_300_T_1_000)
                            .setup(EdidAnalogVideoSetup::BlankLevelIsBlackLevel)
                            .separate_hv_sync_signals(true)
                            .composite_sync_signal_on_hsync(true)
                            .composite_sync_signal_on_green_video(true)
                            .serrations_on_vsync(true)
                            .build(),
                    ))
                    .size(EdidR4ImageSize::Size(
                        EdidScreenSize::builder()
                            .horizontal_cm(EdidScreenSizeLength::try_from(43).unwrap())
                            .vertical_cm(EdidScreenSizeLength::try_from(32).unwrap())
                            .build(),
                    ))
                    .display_transfer_characteristic(
                        EdidDisplayTransferCharacteristics::try_from(2.2).unwrap(),
                    )
                    .feature_support(
                        EdidR4FeatureSupport::builder()
                            .active_off_is_very_low_power(true)
                            .color(EdidR4DisplayColor::Analog(EdidDisplayColorType::RGBColor))
                            .preferred_timing_mode_is_native(true)
                            .continuous_frequency(true)
                            .build(),
                    )
                    .build(),
            )
            .filter_chromaticity(EdidFilterChromaticity::Color(
                EdidChromaticityPoints::builder()
                    .red(EdidChromaticityPoint::try_from((0.627, 0.341)).unwrap())
                    .green(EdidChromaticityPoint::try_from((0.292, 0.605)).unwrap())
                    .blue(EdidChromaticityPoint::try_from((0.149, 0.072)).unwrap())
                    .white(EdidChromaticityPoint::try_from((0.283, 0.297)).unwrap())
                    .build(),
            ))
            .established_timings(vec![
                EdidEstablishedTiming::ET_720_400_70hz,
                EdidEstablishedTiming::ET_720_400_88hz,
                EdidEstablishedTiming::ET_640_480_60hz,
                EdidEstablishedTiming::ET_640_480_67hz,
                EdidEstablishedTiming::ET_640_480_72hz,
                EdidEstablishedTiming::ET_640_480_75hz,
                EdidEstablishedTiming::ET_800_600_56hz,
                EdidEstablishedTiming::ET_800_600_60hz,
                EdidEstablishedTiming::ET_800_600_72hz,
                EdidEstablishedTiming::ET_800_600_75hz,
                EdidEstablishedTiming::ET_832_624_75hz,
                EdidEstablishedTiming::ET_1024_768_87hz_Interlaced,
                EdidEstablishedTiming::ET_1024_768_60hz,
                EdidEstablishedTiming::ET_1024_768_70hz,
                EdidEstablishedTiming::ET_1024_768_75hz,
                EdidEstablishedTiming::ET_1280_1024_75hz,
                EdidEstablishedTiming::ET_1152_870_75hz,
            ])
            .standard_timings(vec![
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1600).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_4_3)
                    .frequency(EdidStandardTimingRefreshRate::try_from(85).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1600).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_4_3)
                    .frequency(EdidStandardTimingRefreshRate::try_from(75).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1600).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_4_3)
                    .frequency(EdidStandardTimingRefreshRate::try_from(70).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1600).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_4_3)
                    .frequency(EdidStandardTimingRefreshRate::try_from(65).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1280).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_5_4)
                    .frequency(EdidStandardTimingRefreshRate::try_from(85).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1280).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_5_4)
                    .frequency(EdidStandardTimingRefreshRate::try_from(60).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(1024).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_4_3)
                    .frequency(EdidStandardTimingRefreshRate::try_from(85).unwrap())
                    .build(),
                EdidStandardTiming::builder()
                    .x(EdidStandardTimingHorizontalSize::try_from(800).unwrap())
                    .ratio(EdidStandardTimingRatio::Ratio_4_3)
                    .frequency(EdidStandardTimingRefreshRate::try_from(85).unwrap())
                    .build(),
            ])
            .descriptors(vec![
                EdidR4Descriptor::DetailedTiming(
                    EdidDescriptorDetailedTiming::builder()
                        .pixel_clock(EdidDetailedTimingPixelClock::try_from(162_000).unwrap())
                        .horizontal_addressable(EdidDescriptor12BitsTiming::try_from(1600).unwrap())
                        .horizontal_blanking(EdidDescriptor12BitsTiming::try_from(560).unwrap())
                        .vertical_addressable(EdidDescriptor12BitsTiming::try_from(1200).unwrap())
                        .vertical_blanking(EdidDescriptor12BitsTiming::try_from(50).unwrap())
                        .horizontal_front_porch(EdidDescriptor10BitsTiming::try_from(64).unwrap())
                        .horizontal_sync_pulse(EdidDescriptor10BitsTiming::try_from(192).unwrap())
                        .vertical_front_porch(EdidDescriptor6BitsTiming::try_from(1).unwrap())
                        .vertical_sync_pulse(EdidDescriptor6BitsTiming::try_from(3).unwrap())
                        .horizontal_size(EdidDetailedTimingSizeMm::try_from(427).unwrap())
                        .vertical_size(EdidDetailedTimingSizeMm::try_from(320).unwrap())
                        .horizontal_border(EdidDescriptor8BitsTiming::try_from(0).unwrap())
                        .vertical_border(EdidDescriptor8BitsTiming::try_from(0).unwrap())
                        .interlace(false)
                        .stereo(EdidDetailedTimingStereo::None)
                        .sync_type(EdidDetailedTimingSync::Digital(
                            EdidDetailedTimingDigitalSync::builder()
                                .kind(EdidDetailedTimingDigitalSyncKind::Separate(
                                    EdidDetailedTimingDigitalSeparateSync::builder()
                                        .vsync_positive(true)
                                        .build(),
                                ))
                                .hsync_positive(true)
                                .build(),
                        ))
                        .build(),
                ),
                EdidR4Descriptor::DisplayRangeLimits(
                    EdidR4DisplayRangeLimits::builder()
                        .min_vfreq(EdidR4DisplayRangeVerticalFreq::try_from(50).unwrap())
                        .max_vfreq(EdidR4DisplayRangeVerticalFreq::try_from(90).unwrap())
                        .min_hfreq(EdidR4DisplayRangeHorizontalFreq::try_from(30).unwrap())
                        .max_hfreq(EdidR4DisplayRangeHorizontalFreq::try_from(110).unwrap())
                        .max_pixelclock(EdidDisplayRangePixelClock::try_from(230).unwrap())
                        .timings_support(EdidR4DisplayRangeVideoTimingsSupport::CVTSupported(
                            EdidR4DisplayRangeVideoTimingsCVT::R1(
                                EdidR4DisplayRangeVideoTimingsCVTR1::builder()
                                    .maximum_active_pixels_per_line(1600)
                                    .supported_aspect_ratios(vec![
                                        EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_4_3,
                                        EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_5_4,
                                    ])
                                    .preferred_aspect_ratio(
                                        EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_4_3,
                                    )
                                    .standard_cvt_blanking_supported(true)
                                    .horizontal_stretch_supported(true)
                                    .vertical_stretch_supported(true)
                                    .preferred_vertical_refresh_rate(
                                        EdidDisplayRangeVerticalFreq::try_from(60).unwrap(),
                                    )
                                    .build(),
                            ),
                        ))
                        .build(),
                ),
                EdidR4Descriptor::EstablishedTimings(
                    EdidR4DescriptorEstablishedTimings::builder()
                        .established_timings(vec![
                            EdidR4DescriptorEstablishedTimingsIII::ET_640_350_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_640_400_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_720_400_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_640_480_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_800_600_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1024_768_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1152_864_75Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1280_960_60Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1280_960_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1280_1024_60Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1280_1024_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_60Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_75Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_85Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_60Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_65Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_70Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_75Hz,
                            EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_85Hz,
                        ])
                        .build(),
                ),
                EdidR4Descriptor::ProductName(EdidDescriptorString::try_from("ABC LCD21").unwrap()),
            ])
            .build();

        assert_eq!(
            edid.into_bytes(),
            &[
                0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, // Header
                0x04, 0x43, // Manufacturer
                0x06, 0xf2, // Product Code
                0x01, 0x00, 0x00, 0x00, // Serial Number
                0x01, 0x11, // Date
                0x01, 0x04, // Version
                0x0f, // Video Input Definition
                0x2b, 0x20, // Size
                0x78, // Display Gamma
                0x2b, // Feature Support Byte
                0x9c, 0x68, 0xa0, 0x57, 0x4a, 0x9b, 0x26, 0x12, 0x48,
                0x4c, // Chromaticity Coordinates
                0xff, 0xff, 0x80, // Established Timings
                // ---------------------------Standard Timings -----------------------------------
                0xa9, 0x59, 0xa9, 0x4f, 0xa9, 0x4a, 0xa9, 0x45, 0x81, 0x99, 0x81, 0x80, 0x61, 0x59,
                0x45, 0x59,
                // ------------------------ Detailed Timing Block --------------------------------
                0x48, 0x3f, 0x40, 0x30, 0x62, 0xb0, 0x32, 0x40, 0x40, 0xc0, 0x13, 0x00, 0xab, 0x40,
                0x11, 0x00, 0x00, 0x1e,
                // ------------------------ Display Range Limits ---------------------------------
                0x00, 0x00, 0x00, 0xfd, 0x00, 0x32, 0x5a, 0x1e, 0x6e, 0x17, 0x04, 0x11, 0x00, 0xc8,
                0x90, 0x08, // <- The spec example got that byte wrong (I think?)
                0x50, 0x3c,
                // --------------------Established Timings III Block -----------------------------
                0x00, 0x00, 0x00, 0xf7, 0x00, 0x0a, 0xf7, 0x0f, 0x03, 0x87, 0xc0, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
                // ---------------------- Display Product Name -----------------------------------
                0x00, 0x00, 0x00, 0xfc, 0x00, 0x41, 0x42, 0x43, 0x20, 0x4c, 0x43, 0x44, 0x32, 0x31,
                0x0a, 0x20, 0x20, 0x20,
                // -------------------------- Extension Flag -------------------------------------
                0x00,
                // ----------------------------- Checksum ----------------------------------------
                // The checksum isn't valid in the example either. It should be 0x9a, and since
                // some part of the EDID were wrong it's further modified.
                0x92,
            ]
        );
    }
}
