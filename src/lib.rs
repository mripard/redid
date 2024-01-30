use core::{
    array,
    convert::{TryFrom, TryInto},
    fmt, num,
};

use num_traits::ToPrimitive;
use typed_builder::TypedBuilder;

mod descriptors;

pub use descriptors::{
    EdidDescriptor, EdidDescriptorCustom, EdidDescriptorDetailedTiming, EdidDescriptorString,
    EdidDetailedTimingAnalogSync, EdidDetailedTimingDigitalCompositeSync,
    EdidDetailedTimingDigitalSeparateSync, EdidDetailedTimingDigitalSync,
    EdidDetailedTimingDigitalSyncKind, EdidDetailedTimingStereo, EdidDetailedTimingSync,
    EdidDisplayRangeHorizontalFreq, EdidDisplayRangePixelClock, EdidDisplayRangeVerticalFreq,
    EdidDisplayRangeVideoTimingsGTF, EdidDisplayRangeVideoTimingsGTFStartFrequency,
    EdidR3Descriptor, EdidR3DisplayRangeLimits, EdidR3DisplayRangeVideoTimingsSupport,
    EdidR4Descriptor, EdidR4DescriptorEstablishedTimings, EdidR4DescriptorEstablishedTimingsIII,
    EdidR4DisplayRangeHorizontalFreq, EdidR4DisplayRangeLimits, EdidR4DisplayRangeVerticalFreq,
    EdidR4DisplayRangeVideoTimingsAspectRatio, EdidR4DisplayRangeVideoTimingsCVT,
    EdidR4DisplayRangeVideoTimingsCVTR1, EdidR4DisplayRangeVideoTimingsSupport,
};

const EDID_BASE_LEN: usize = 128;
const EDID_MANUFACTURER_LEN: usize = 3;
const EDID_DESCRIPTOR_LEN: usize = 18;
const EDID_DESCRIPTORS_NUM: usize = 4;
const EDID_DESCRIPTOR_PAYLOAD_LEN: usize = 13;

pub trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
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

impl<D: fmt::Display + fmt::Debug> std::error::Error for EdidTypeConversionError<D> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
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
pub struct EdidManufacturer([u8; EDID_MANUFACTURER_LEN]);

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

        if value.len() != EDID_MANUFACTURER_LEN {
            return Err(EdidTypeConversionError::Value(String::from(
                "Manufacturer ID must be 3 characters long.",
            )));
        }

        Ok(Self(value.as_bytes().try_into()?))
    }
}

impl IntoBytes for EdidManufacturer {
    fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2);

        let manufacturer = &self.0;
        let mut comp = (manufacturer[0] - b'@') << 2;
        comp |= (manufacturer[1] - b'@') >> 3;
        bytes.push(comp);

        comp = (manufacturer[1] - b'@') << 5;
        comp |= manufacturer[2] - b'@';
        bytes.push(comp);

        bytes
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
        let mut bytes = Vec::with_capacity(2);

        let prod = &self.0;
        bytes.push((prod & 0xff) as u8);
        bytes.push((prod >> 8) as u8);

        bytes
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
        let mut bytes = Vec::with_capacity(4);

        let serial = &self.0;
        bytes.push((serial & 0xff) as u8);
        bytes.push(((serial >> 8) & 0xff) as u8);
        bytes.push(((serial >> 16) & 0xff) as u8);
        bytes.push(((serial >> 24) & 0xff) as u8);

        bytes
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EdidWeek(u8);

impl TryFrom<u8> for EdidWeek {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=54).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(54)));
        }

        Ok(Self(value))
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

/// EDID Manufacture Date Representation.
///
/// Contains a year, starting from 1990, and an optional week in the 1-54 range.
///
/// # Examples
///
/// It is taken from the EDID 1.4 Specification, Section 3.4.4, Example 1.
///
/// ```
/// use edid::{EdidManufactureDate, IntoBytes};
/// use std::convert::TryInto;
///
/// let date: EdidManufactureDate = (1, 2006).try_into().unwrap();
///
/// assert_eq!(date.into_bytes(), &[0x01, 0x10]);
/// ```
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
        let week = if let Some(val) = self.0 { val.0 } else { 0 };
        let year = u8::try_from(self.1 .0 - 1990).expect("Year would overflow our type.");

        Vec::from(&[week, year])
    }
}

/// EDID 1.4 Model Date Representation.
///
/// Contains a Year, starting from 1990.
///
/// # Examples
///
/// It is taken from the EDID 1.4 Specification, Section 3.4.4, Example 2.
///
/// ```
/// use edid::{EdidR4ModelDate, IntoBytes};
/// use std::convert::TryInto;
///
/// let date: EdidR4ModelDate = 2006.try_into().unwrap();
///
/// assert_eq!(date.into_bytes(), &[0xff, 0x10]);
/// ```
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
        let year = u8::try_from(self.0 .0 - 1990).expect("Year would overflow our type.");

        Vec::from(&[0xff, year])
    }
}

/// EDID 1.4 Date Representation.
///
/// # Examples
///
/// They are taken from the EDID 1.4 Specification, Section 3.4.4.
///
/// ```
/// use edid::{EdidManufactureDate, EdidR4Date, EdidR4ModelDate, IntoBytes};
/// use std::convert::TryInto;
///
/// let manufacture_date: EdidManufactureDate = (1, 2006).try_into().unwrap();
/// let manufacture = EdidR4Date::Manufacture(manufacture_date);
///
/// assert_eq!(manufacture.into_bytes(), &[0x01, 0x10]);
///
/// let model_date: EdidR4ModelDate = 2006.try_into().unwrap();
/// let model = EdidR4Date::Model(model_date);
///
/// assert_eq!(model.into_bytes(), &[0xff, 0x10]);
/// ```
#[derive(Clone, Copy, Debug)]
pub enum EdidR4Date {
    Manufacture(EdidManufactureDate),
    Model(EdidR4ModelDate),
}

impl IntoBytes for EdidR4Date {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            EdidR4Date::Manufacture(m) => m.into_bytes(),
            EdidR4Date::Model(m) => m.into_bytes(),
        }
    }
}

/// EDID Date Representation.
///
/// # Examples
///
/// They are taken from the EDID 1.3 Specification, Section 3.4, and EDID 1.4 Specification, Section
/// 3.4.4.
///
/// ```
/// use edid::{EdidDate, EdidManufactureDate, EdidR4Date, EdidR4ModelDate, IntoBytes};
/// use std::convert::TryInto;
///
/// let manufacture_date: EdidManufactureDate = 1997.try_into().unwrap();
/// let date = EdidDate::R3(manufacture_date);
///
/// assert_eq!(date.into_bytes(), &[0x00, 0x07]);
///
/// let manufacture_date: EdidManufactureDate = (1, 2006).try_into().unwrap();
/// let manufacture = EdidR4Date::Manufacture(manufacture_date);
/// let date = EdidDate::R4(manufacture);
///
/// assert_eq!(date.into_bytes(), &[0x01, 0x10]);
///
/// let model_date: EdidR4ModelDate = 2006.try_into().unwrap();
/// let model = EdidR4Date::Model(model_date);
/// let date = EdidDate::R4(model);
///
/// assert_eq!(date.into_bytes(), &[0xff, 0x10]);
/// ```
#[derive(Clone, Copy, Debug)]
pub enum EdidDate {
    R3(EdidManufactureDate),
    R4(EdidR4Date),
}

impl IntoBytes for EdidDate {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            EdidDate::R3(v) => v.into_bytes(),
            EdidDate::R4(v) => v.into_bytes(),
        }
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

        Vec::from(&[byte])
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
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR3VideoInputDefinition {
    Analog(EdidAnalogVideoInputDefinition),
    Digital(EdidR3DigitalVideoInputDefinition),
}

impl IntoBytes for EdidR3VideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Analog(v) => v.into_bytes(),
            Self::Digital(v) => v.into_bytes(),
        }
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
pub struct EdidScreenSize {
    // FIXME: 0 is an invalid value
    horizontal_cm: u8,

    // FIXME: 0 is an invalid value
    vertical_cm: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR3ImageSize {
    Size(EdidScreenSize),
    Undefined,
}

impl IntoBytes for EdidR3ImageSize {
    fn into_bytes(self) -> Vec<u8> {
        let bytes = match self {
            Self::Size(s) => [s.horizontal_cm, s.vertical_cm],
            Self::Undefined => [0x00, 0x00],
        };

        Vec::from(&bytes)
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

        Vec::from(&[stored])
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
    preferred_timing_mode_is_first_detailed_timing_block: bool,

    #[builder(default)]
    default_gtf_supported: bool,
}

impl IntoBytes for EdidR3FeatureSupport {
    fn into_bytes(self) -> Vec<u8> {
        let mut byte = 0;

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

        if self.preferred_timing_mode_is_first_detailed_timing_block {
            byte |= 1 << 1;
        }

        if self.default_gtf_supported {
            byte |= 1 << 0;
        }

        Vec::from(&[byte])
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
        let mut bytes = Vec::with_capacity(5);

        bytes.extend_from_slice(&self.video_input.into_bytes());
        bytes.extend_from_slice(&self.size.into_bytes());
        bytes.extend_from_slice(&self.display_transfer_characteristic.into_bytes());
        bytes.extend_from_slice(&self.feature_support.into_bytes());

        bytes
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

        Vec::from(&[byte])
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidR4VideoInputDefinition {
    Analog(EdidAnalogVideoInputDefinition),
    Digital(EdidR4DigitalVideoInputDefinition),
}

impl IntoBytes for EdidR4VideoInputDefinition {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Analog(v) => v.into_bytes(),
            Self::Digital(v) => v.into_bytes(),
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
        let bytes = match self {
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
            Self::Size(s) => [s.horizontal_cm, s.vertical_cm],
            Self::Undefined => [0x00, 0x00],
        };

        Vec::from(&bytes)
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
    standby: bool,

    #[builder(default)]
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

        if self.standby {
            byte |= 1 << 7;
        }

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

        Vec::from(&[byte])
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
        let mut bytes = Vec::with_capacity(5);

        bytes.extend_from_slice(&self.video_input.into_bytes());
        bytes.extend_from_slice(&self.size.into_bytes());
        bytes.extend_from_slice(&self.display_transfer_characteristic.into_bytes());
        bytes.extend_from_slice(&self.feature_support.into_bytes());

        bytes
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidBasicDisplayParametersFeatures {
    R3(EdidR3BasicDisplayParametersFeatures),
    R4(EdidR4BasicDisplayParametersFeatures),
}

impl IntoBytes for EdidBasicDisplayParametersFeatures {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::R3(v) => v.into_bytes(),
            Self::R4(v) => v.into_bytes(),
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
        assert_eq!(EdidChromaticityCoordinate(0.610).into_raw(), 0b1001110001);
        assert_eq!(EdidChromaticityCoordinate(0.307).into_raw(), 0b0100111010);
        assert_eq!(EdidChromaticityCoordinate(0.150).into_raw(), 0b0010011010);
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

        Vec::from(&bytes)
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
            };
        }

        Vec::from(&[byte0, byte1, byte2])
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
        let mut bytes = Vec::with_capacity(16);

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
                None => bytes.extend_from_slice(&[1, 1]),
            };
        }

        bytes
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

        // FIXME: Support the extensions
        bytes.push(0);

        let mut sum: u8 = 0;
        for byte in &bytes {
            sum = sum.wrapping_add(*byte);
        }

        let checksum = 0u8.wrapping_sub(sum);
        bytes.push(checksum);

        assert_eq!(
            bytes.len(),
            EDID_BASE_LEN,
            "EDID is larger than it should ({} vs expected {} bytes",
            bytes.len(),
            EDID_BASE_LEN
        );
        bytes
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
        }
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(mutators(
    pub fn descriptors(&mut self, d: Vec<EdidR3Descriptor>) {
        self.descriptors = d.into_iter().map(EdidDescriptor::R3).collect();
    }

    pub fn add_descriptor(&mut self, d: EdidR3Descriptor) {
        self.descriptors.push(EdidDescriptor::R3(d));
    }

    pub fn established_timings(&mut self, et: Vec<EdidEstablishedTiming>) {
        self.established_timings = et;
    }

    pub fn add_established_timing(&mut self, et: EdidEstablishedTiming) {
        self.established_timings.push(et);
    }

    pub fn standard_timings(&mut self, st: Vec<EdidStandardTiming>) {
        self.standard_timings = st;
    }

    pub fn add_standard_timing(&mut self, st: EdidStandardTiming) {
        self.standard_timings.push(st);
    }
))]
pub struct EdidRelease3 {
    manufacturer: EdidManufacturer,

    #[builder(setter(into))]
    product_code: EdidProductCode,

    #[builder(default, setter(into))]
    serial_number: Option<EdidSerialNumber>,

    date: EdidManufactureDate,
    display_parameters_features: EdidR3BasicDisplayParametersFeatures,
    filter_chromaticity: EdidFilterChromaticity,

    #[builder(via_mutators)]
    established_timings: Vec<EdidEstablishedTiming>,

    #[builder(via_mutators)]
    standard_timings: Vec<EdidStandardTiming>,

    // FIXME: The Preferred Timing Descriptors is required in the first position
    // FIXME: Monitor Name is mandatory
    // FIXME: Display Range Limits is mandatory
    #[builder(via_mutators)]
    descriptors: Vec<EdidDescriptor>,
}

impl IntoBytes for EdidRelease3 {
    fn into_bytes(self) -> Vec<u8> {
        Edid::from(self).into_bytes()
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[builder(mutators(
    pub fn descriptors(&mut self, d: Vec<EdidR4Descriptor>) {
        self.descriptors = d.into_iter().map(EdidDescriptor::R4).collect();
    }

    pub fn add_descriptor(&mut self, d: EdidR4Descriptor) {
        self.descriptors.push(EdidDescriptor::R4(d));
    }

    pub fn established_timings(&mut self, et: Vec<EdidEstablishedTiming>) {
        self.established_timings = et;
    }

    pub fn add_established_timing(&mut self, et: EdidEstablishedTiming) {
        self.established_timings.push(et);
    }

    pub fn standard_timings(&mut self, st: Vec<EdidStandardTiming>) {
        self.standard_timings = st;
    }

    pub fn add_standard_timing(&mut self, st: EdidStandardTiming) {
        self.standard_timings.push(st);
    }
))]
pub struct EdidRelease4 {
    manufacturer: EdidManufacturer,

    #[builder(setter(into))]
    product_code: EdidProductCode,

    #[builder(default, setter(into))]
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
}

impl IntoBytes for EdidRelease4 {
    fn into_bytes(self) -> Vec<u8> {
        Edid::from(self).into_bytes()
    }
}
