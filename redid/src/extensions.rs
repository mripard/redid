use num_traits::ToPrimitive as _;
#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer};
use typed_builder::TypedBuilder;

use crate::{EdidDescriptorDetailedTiming, EdidTypeConversionError, IntoBytes};

const EDID_EXTENSION_CTA_861_LEN: usize = 128;

const EDID_EXTENSION_CTA_861_DATA_BLOCK_HEADER_LEN: usize = 1;
const EDID_EXTENSION_CTA_861_AUDIO_DESCRIPTOR_LEN: usize = 3;
const EDID_EXTENSION_CTA_861_VIDEO_DESCRIPTOR_LEN: usize = 1;
const EDID_EXTENSION_CTA_861_VENDOR_OUI_LEN: usize = 3;
const EDID_EXTENSION_CTA_861_VENDOR_HEADER_LEN: usize =
    EDID_EXTENSION_CTA_861_DATA_BLOCK_HEADER_LEN + EDID_EXTENSION_CTA_861_VENDOR_OUI_LEN;
const EDID_EXTENSION_CTA_861_SPEAKER_ALLOCATION_LEN: usize =
    EDID_EXTENSION_CTA_861_DATA_BLOCK_HEADER_LEN + 3;
// TODO: VESA Display Transfer Characteristic Data Block

const EDID_EXTENSION_CTA_861_DATA_BLOCK_EXTENDED_HEADER_LEN: usize = 2;
const EDID_EXTENSION_CTA_861_VIDEO_CAPABILITY_LEN: usize =
    EDID_EXTENSION_CTA_861_DATA_BLOCK_EXTENDED_HEADER_LEN + 1;
const EDID_EXTENSION_CTA_861_COLORIMETRY_LEN: usize =
    EDID_EXTENSION_CTA_861_DATA_BLOCK_EXTENDED_HEADER_LEN + 2;

const EDID_EXTENSION_CTA_861_HDMI_HEADER_LEN: usize = EDID_EXTENSION_CTA_861_VENDOR_HEADER_LEN + 2;
const EDID_EXTENSION_CTA_861_HDMI_VIDEO_HEADER_LEN: usize = 2;

const EDID_EXTENSION_CTA_861_HDR_STATIC_METADATA_HEADER_LEN: usize =
    EDID_EXTENSION_CTA_861_DATA_BLOCK_EXTENDED_HEADER_LEN + 2;

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u8"))]
pub struct EdidExtensionCTA861AudioDataBlockChannels(u8);

impl TryFrom<u8> for EdidExtensionCTA861AudioDataBlockChannels {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=8).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(1), Some(8)));
        }

        Ok(Self(value))
    }
}

#[allow(clippy::enum_variant_names)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum EdidExtensionCTA861AudioDataBlockSamplingFrequency {
    Frequency32kHz = 0,
    Frequency44_1kHz,
    Frequency48kHz,
    Frequency88_2kHz,
    Frequency96kHz,
    Frequency176_4kHz,
    Frequency192kHz,
}

#[allow(clippy::enum_variant_names)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum EdidExtensionCTA861AudioDataBlockSamplingRate {
    Rate16Bit = 0,
    Rate20Bit,
    Rate24Bit,
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn sampling_frequencies(&mut self, freqs: Vec<EdidExtensionCTA861AudioDataBlockSamplingFrequency>) {
        self.sampling_frequencies = freqs;
    }

    #[allow(unreachable_pub)]
    pub fn add_sampling_frequency(&mut self, freq: EdidExtensionCTA861AudioDataBlockSamplingFrequency) {
        self.sampling_frequencies.push(freq);
    }

    #[allow(unreachable_pub)]
    pub fn sampling_rates(&mut self, rates: Vec<EdidExtensionCTA861AudioDataBlockSamplingRate>) {
        self.sampling_rates = rates;
    }

    #[allow(unreachable_pub)]
    pub fn add_sampling_rate(&mut self, rate: EdidExtensionCTA861AudioDataBlockSamplingRate) {
        self.sampling_rates.push(rate);
    }
))]
pub struct EdidExtensionCTA861AudioDataBlockLPCM {
    channels: EdidExtensionCTA861AudioDataBlockChannels,

    #[builder(via_mutators)]
    sampling_frequencies: Vec<EdidExtensionCTA861AudioDataBlockSamplingFrequency>,

    #[builder(via_mutators)]
    sampling_rates: Vec<EdidExtensionCTA861AudioDataBlockSamplingRate>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum EdidExtensionCTA861AudioDataBlockDesc {
    #[allow(clippy::upper_case_acronyms)]
    LPCM(EdidExtensionCTA861AudioDataBlockLPCM),
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn descriptors(&mut self, desc: Vec<EdidExtensionCTA861AudioDataBlockDesc>) {
        self.desc = desc;
    }

    #[allow(unreachable_pub)]
    pub fn add_short_audio_descriptor(&mut self, desc: EdidExtensionCTA861AudioDataBlockDesc) {
        self.desc.push(desc);
    }
))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861AudioDataBlock {
    #[builder(via_mutators)]
    desc: Vec<EdidExtensionCTA861AudioDataBlockDesc>,
}

impl IntoBytes for EdidExtensionCTA861AudioDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size());

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");
        data.push(1 << 5 | size);

        for desc in &self.desc {
            match desc {
                EdidExtensionCTA861AudioDataBlockDesc::LPCM(b) => {
                    let byte0 = 1 << 3 | (b.channels.0 - 1);

                    let mut byte1 = 0;
                    for f in &b.sampling_frequencies {
                        byte1 |= 1 << (*f as u8);
                    }

                    let mut byte2 = 0;
                    for r in &b.sampling_rates {
                        byte2 |= 1 << (*r as u8);
                    }

                    data.extend_from_slice(&[byte0, byte1, byte2]);
                }
            }
        }

        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_DATA_BLOCK_HEADER_LEN
            + (EDID_EXTENSION_CTA_861_AUDIO_DESCRIPTOR_LEN * self.desc.len())
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(field_defaults(setter(strip_bool)))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861SpeakerAllocationDataBlock {
    front_left_front_right: bool,
    low_frequency_effects: bool,
    front_center: bool,
    back_left_back_right: bool,
    back_center: bool,
    front_left_of_center_front_right_of_center: bool,
    rear_left_of_center_rear_right_of_center: bool,
    front_left_wide_front_right_wide: bool,
    top_front_left_top_front_right: bool,
    top_center: bool,
    top_front_center: bool,
    left_surround_right_surround: bool,
    low_frequency_effects_2: bool,
    top_back_center: bool,
    side_left_side_right: bool,
    top_side_left_top_side_right: bool,
    top_back_left_top_back_right: bool,
    bottom_front_center: bool,
    bottom_from_left_bottom_front_right: bool,
    top_left_surround_top_right_surround: bool,
}

impl IntoBytes for EdidExtensionCTA861SpeakerAllocationDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(EDID_EXTENSION_CTA_861_SPEAKER_ALLOCATION_LEN);

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");
        data.push(4 << 5 | size);

        let mut byte = 0;
        if self.front_left_wide_front_right_wide {
            byte |= 1 << 7;
        }

        if self.rear_left_of_center_rear_right_of_center {
            byte |= 1 << 6;
        }

        if self.front_left_of_center_front_right_of_center {
            byte |= 1 << 5;
        }

        if self.back_center {
            byte |= 1 << 4;
        }

        if self.back_left_back_right {
            byte |= 1 << 3;
        }

        if self.front_center {
            byte |= 1 << 2;
        }

        if self.low_frequency_effects {
            byte |= 1 << 1;
        }

        if self.front_left_front_right {
            byte |= 1;
        }
        data.push(byte);

        let mut byte = 0;
        if self.top_side_left_top_side_right {
            byte |= 1 << 7;
        }

        if self.side_left_side_right {
            byte |= 1 << 6;
        }

        if self.top_back_center {
            byte |= 1 << 5;
        }

        if self.low_frequency_effects_2 {
            byte |= 1 << 4;
        }

        if self.left_surround_right_surround {
            byte |= 1 << 3;
        }

        if self.top_front_center {
            byte |= 1 << 2;
        }

        if self.top_center {
            byte |= 1 << 1;
        }

        if self.top_front_left_top_front_right {
            byte |= 1;
        }
        data.push(byte);

        let mut byte = 0;
        if self.top_left_surround_top_right_surround {
            byte |= 1 << 3;
        }

        if self.bottom_from_left_bottom_front_right {
            byte |= 1 << 2;
        }

        if self.bottom_front_center {
            byte |= 1 << 1;
        }

        if self.top_back_left_top_back_right {
            byte |= 1;
        }
        data.push(byte);

        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_SPEAKER_ALLOCATION_LEN
    }
}

#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861ColorimetryDataBlock {
    xv_ycc_601: bool,
    xv_ycc_709: bool,
    s_ycc_601: bool,
    op_ycc_601: bool,
    op_rgb: bool,
    bt_2020_c_ycc: bool,
    bt_2020_ycc: bool,
    bt_2020_rgb: bool,
    dci_p3: bool,
}

impl IntoBytes for EdidExtensionCTA861ColorimetryDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(EDID_EXTENSION_CTA_861_COLORIMETRY_LEN);

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");
        data.push(7 << 5 | size);
        data.push(5);

        let mut byte = 0;
        if self.bt_2020_rgb {
            byte |= 1 << 7;
        }

        if self.bt_2020_ycc {
            byte |= 1 << 6;
        }

        if self.bt_2020_c_ycc {
            byte |= 1 << 5;
        }

        if self.op_rgb {
            byte |= 1 << 4;
        }

        if self.op_ycc_601 {
            byte |= 1 << 3;
        }

        if self.s_ycc_601 {
            byte |= 1 << 2;
        }

        if self.xv_ycc_709 {
            byte |= 1 << 1;
        }

        if self.xv_ycc_601 {
            byte |= 1 << 0;
        }

        data.push(byte);

        // FIXME: introduced by CTA-861.6 and made mandatory by edid-check
        let mut byte = 1 << 5;
        if self.dci_p3 {
            byte |= 1 << 7;
        }

        data.push(byte);

        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_COLORIMETRY_LEN
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EdidExtensionCTA861VideoDataBlockDesc {
    Low(bool, u8),
    High(u8),
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for EdidExtensionCTA861VideoDataBlockDesc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct VicHelper {
            vic: u8,
            #[serde(default)]
            native: Option<bool>,
        }

        let helper = VicHelper::deserialize(deserializer)?;

        if helper.vic < 64 {
            if let Some(native) = helper.native {
                Ok(Self::Low(native, helper.vic))
            } else {
                Ok(Self::Low(false, helper.vic))
            }
        } else if helper.native.is_none() {
            Ok(Self::High(helper.vic))
        } else {
            Err(de::Error::custom("VICs > 64 cannot be native."))
        }
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn descriptors(&mut self, desc: Vec<EdidExtensionCTA861VideoDataBlockDesc>) {
        self.descriptors = desc;
    }

    #[allow(unreachable_pub)]
    pub fn add_short_video_descriptor(&mut self, vic: u8) {
        self.descriptors.push(if vic < 64 {
            EdidExtensionCTA861VideoDataBlockDesc::Low(false, vic)
        } else {
            EdidExtensionCTA861VideoDataBlockDesc::High(vic)
        });
    }

    #[allow(unreachable_pub)]
    pub fn add_native_short_video_descriptor(&mut self, vic: u8) {
        self.descriptors
            .push(EdidExtensionCTA861VideoDataBlockDesc::Low(true, vic));
    }
))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861VideoDataBlock {
    #[builder(via_mutators)]
    descriptors: Vec<EdidExtensionCTA861VideoDataBlockDesc>,
}

impl IntoBytes for EdidExtensionCTA861VideoDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size());

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");

        data.push(2 << 5 | size);

        for desc in &self.descriptors {
            match desc {
                EdidExtensionCTA861VideoDataBlockDesc::Low(native, vic) => {
                    let byte = if *native { 1 << 7 | vic } else { *vic };

                    data.push(byte);
                }
                EdidExtensionCTA861VideoDataBlockDesc::High(vic) => {
                    data.push(*vic);
                }
            }
        }

        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_DATA_BLOCK_HEADER_LEN
            + (EDID_EXTENSION_CTA_861_VIDEO_DESCRIPTOR_LEN * self.descriptors.len())
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String"))]
pub struct CecAddress(u8, u8, u8, u8);

impl TryFrom<[u8; 4]> for CecAddress {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        for val in value {
            if !(0..=15).contains(&val) {
                return Err(EdidTypeConversionError::Range(val, None, Some(15)));
            }
        }

        Ok(Self(value[0], value[1], value[2], value[3]))
    }
}

impl TryFrom<String> for CecAddress {
    type Error = EdidTypeConversionError<u8>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let groups: Vec<&str> = value.split('.').collect();
        let mut res = [0; 4];

        if groups.len() != 4 {
            return Err(EdidTypeConversionError::Value(value));
        }

        for (idx, slot) in res.iter_mut().enumerate() {
            *slot = groups[idx]
                .parse()
                .map_err(|_e| EdidTypeConversionError::Value(groups[idx].to_owned()))?;
        }

        res.try_into()
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "u16"))]
pub struct EdidExtensionCTA861Hdmi14bTmdsRate(u16);

impl TryFrom<u16> for EdidExtensionCTA861Hdmi14bTmdsRate {
    type Error = EdidTypeConversionError<u16>;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(5..=340).contains(&value) {
            return Err(EdidTypeConversionError::Range(value, Some(5), Some(340)));
        }

        Ok(Self(value))
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn add_vic(&mut self, vic: u8) {
        self.vics.push(vic);
    }

    #[allow(unreachable_pub)]
    pub fn vics(&mut self, vics: Vec<u8>) {
        self.vics = vics;
    }
))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861Hdmi14bDataBlockVideo {
    #[builder(via_mutators)]
    vics: Vec<u8>,
    // FIXME: Handle Image Size attributes
    // FIXME: Handle 3d
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861HdmiDataBlock {
    source_physical_address: CecAddress,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    deep_color_30_bits: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    deep_color_36_bits: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    deep_color_48_bits: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    deep_color_ycbcr_444: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    dvi_dual: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    acp_isrc: bool,

    #[builder(default, setter(strip_option))]
    #[cfg_attr(feature = "serde", serde(default))]
    max_tmds_rate_mhz: Option<EdidExtensionCTA861Hdmi14bTmdsRate>,

    #[builder(default, setter(strip_option))]
    #[cfg_attr(feature = "serde", serde(default))]
    video: Option<EdidExtensionCTA861Hdmi14bDataBlockVideo>,
    // FIXME: Handle CNC
    // FIXME: Handle latencies
}

impl IntoBytes for EdidExtensionCTA861HdmiDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size());

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");

        data.push(3 << 5 | size);
        data.extend_from_slice(&[0x03u8, 0x0cu8, 0x00u8]);

        data.push(self.source_physical_address.0 << 4 | self.source_physical_address.1);
        data.push(self.source_physical_address.2 << 4 | self.source_physical_address.3);

        // FIXME: Handle latencies and CNC
        if self.video.is_some() {
            data.resize(9, 0);
        } else if self.max_tmds_rate_mhz.is_some() {
            data.resize(8, 0);
        } else if self.acp_isrc
            || self.deep_color_30_bits
            || self.deep_color_36_bits
            || self.deep_color_48_bits
            || self.deep_color_ycbcr_444
            || self.dvi_dual
        {
            data.resize(7, 0);
        }

        // Byte 6
        if data.len() > 6 {
            let mut byte = 0;
            if self.acp_isrc {
                byte |= 1 << 7;
            }

            if self.deep_color_48_bits {
                byte |= 1 << 6;
            }

            if self.deep_color_36_bits {
                byte |= 1 << 5;
            }

            if self.deep_color_30_bits {
                byte |= 1 << 4;
            }

            if self.deep_color_ycbcr_444 {
                byte |= 1 << 3;
            }

            if self.dvi_dual {
                byte |= 1;
            }

            data[6] = byte;
        }

        // Byte 7
        if data.len() > 7 {
            let mut byte = 0;

            if let Some(val) = self.max_tmds_rate_mhz {
                let rate = val
                    .0
                    .div_ceil(5)
                    .to_u8()
                    .expect("Rate would overflow our type");

                byte = rate;
            }

            data[7] = byte;
        }

        // Byte 8
        if data.len() > 8 {
            let mut byte = 0;

            if self.video.is_some() {
                byte |= 1 << 5;
            }

            // FIXME: Handle latencies

            data[8] = byte;
        }

        // FIXME: Handle latencies

        if let Some(val) = self.video {
            // FIXME: Handle 3D and Image Size attributes
            data.push(0);

            let vics = val
                .vics
                .len()
                .to_u8()
                .expect("Number of VICs would overflow our type.");
            data.push(vics << 5);

            for vic in &val.vics {
                data.push(*vic);
            }

            // FIXME: Handle 3d
        }

        data
    }

    fn size(&self) -> usize {
        let mut size = EDID_EXTENSION_CTA_861_HDMI_HEADER_LEN;

        // FIXME: Handle latencies and CNC
        if self.video.is_some() {
            size += 3;
        } else if self.max_tmds_rate_mhz.is_some() {
            size += 2;
        } else if self.acp_isrc
            || self.deep_color_30_bits
            || self.deep_color_36_bits
            || self.deep_color_48_bits
            || self.deep_color_ycbcr_444
            || self.dvi_dual
        {
            size += 1;
        }

        // FIXME: Handle latencies

        if let Some(val) = &self.video {
            size += EDID_EXTENSION_CTA_861_HDMI_VIDEO_HEADER_LEN;
            size += val.vics.len();

            // FIXME: Handle 3d
        }

        size
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidExtensionCTA861VideoCapabilityQuantization {
    NoData,
    Selectable,
}

impl EdidExtensionCTA861VideoCapabilityQuantization {
    fn as_raw(self) -> u8 {
        match self {
            EdidExtensionCTA861VideoCapabilityQuantization::NoData => 0,
            EdidExtensionCTA861VideoCapabilityQuantization::Selectable => 1,
        }
    }
}

impl Default for EdidExtensionCTA861VideoCapabilityQuantization {
    fn default() -> Self {
        Self::NoData
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidExtensionCTA861VideoCapabilityScanBehavior {
    NotSupported,
    Overscanned,
    Underscanned,
    Both,
}

impl Default for EdidExtensionCTA861VideoCapabilityScanBehavior {
    fn default() -> Self {
        Self::NotSupported
    }
}

impl EdidExtensionCTA861VideoCapabilityScanBehavior {
    fn as_raw(self) -> u8 {
        match self {
            EdidExtensionCTA861VideoCapabilityScanBehavior::NotSupported => 0,
            EdidExtensionCTA861VideoCapabilityScanBehavior::Overscanned => 1,
            EdidExtensionCTA861VideoCapabilityScanBehavior::Underscanned => 2,
            EdidExtensionCTA861VideoCapabilityScanBehavior::Both => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(field_defaults(default))]
#[cfg_attr(feature = "serde", serde(default))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861VideoCapabilityDataBlock {
    qy_quant: EdidExtensionCTA861VideoCapabilityQuantization,
    qs_quant: EdidExtensionCTA861VideoCapabilityQuantization,
    pt_scan: EdidExtensionCTA861VideoCapabilityScanBehavior,
    it_scan: EdidExtensionCTA861VideoCapabilityScanBehavior,
    ce_scan: EdidExtensionCTA861VideoCapabilityScanBehavior,
}

impl IntoBytes for EdidExtensionCTA861VideoCapabilityDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(EDID_EXTENSION_CTA_861_VIDEO_CAPABILITY_LEN);

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");

        data.push(7 << 5 | size);
        data.push(0);

        let mut byte = 0;
        byte |= self.qy_quant.as_raw() << 7;
        byte |= self.qs_quant.as_raw() << 6;
        byte |= self.pt_scan.as_raw() << 4;
        byte |= self.it_scan.as_raw() << 2;
        byte |= self.ce_scan.as_raw();

        data.push(byte);

        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_VIDEO_CAPABILITY_LEN
    }
}

/// Static metadata descriptor types advertised by CTA-861-G, Section 7.5.13.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidExtensionCTA861HdrStaticMetadataDescriptor {
    /// Static Metadata Type 1 (CTA-861-G, Section 6.9.1, Table 45).
    Type1 = 0,
}

/// Desired content luminance code values from CTA-861-G, Section 7.5.13.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "snake_case", deny_unknown_fields)
)]
pub enum EdidExtensionCTA861HdrStaticMetadataLuminance {
    #[default]
    None,
    Max {
        max_luminance: u8,
    },
    MaxAvg {
        max_luminance: u8,
        max_avg_luminance: u8,
    },
    MinMaxAvg {
        max_luminance: u8,
        max_avg_luminance: u8,
        min_luminance: u8,
    },
}

impl EdidExtensionCTA861HdrStaticMetadataLuminance {
    fn append_to(self, data: &mut Vec<u8>) {
        match self {
            Self::None => {}
            Self::Max { max_luminance } => data.push(max_luminance),
            Self::MaxAvg {
                max_luminance,
                max_avg_luminance,
            } => data.extend_from_slice(&[max_luminance, max_avg_luminance]),
            Self::MinMaxAvg {
                max_luminance,
                max_avg_luminance,
                min_luminance,
            } => data.extend_from_slice(&[max_luminance, max_avg_luminance, min_luminance]),
        }
    }

    fn size(self) -> usize {
        match self {
            Self::None => 0,
            Self::Max { .. } => 1,
            Self::MaxAvg { .. } => 2,
            Self::MinMaxAvg { .. } => 3,
        }
    }
}

/// HDR Static Metadata Data Block from CTA-861-G, Section 7.5.13.
#[derive(Clone, Copy, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861HdrStaticMetadataDataBlock {
    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    traditional_gamma_sdr: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    traditional_gamma_hdr: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    smpte_st2084: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    bt_2100_hlg: bool,

    #[builder(default, setter(strip_option))]
    #[cfg_attr(feature = "serde", serde(default))]
    static_metadata_descriptor: Option<EdidExtensionCTA861HdrStaticMetadataDescriptor>,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    luminance: EdidExtensionCTA861HdrStaticMetadataLuminance,
}

impl IntoBytes for EdidExtensionCTA861HdrStaticMetadataDataBlock {
    fn into_bytes(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.size());

        let size = (self.size() - 1)
            .to_u8()
            .expect("Size would overflow our type");

        data.push(7 << 5 | size);
        data.push(6);

        // EOTF support bits
        let mut byte = 0;
        if self.traditional_gamma_sdr {
            byte |= 1 << 0;
        }
        if self.traditional_gamma_hdr {
            byte |= 1 << 1;
        }
        if self.smpte_st2084 {
            byte |= 1 << 2;
        }
        if self.bt_2100_hlg {
            byte |= 1 << 3;
        }
        data.push(byte);

        // Static metadata descriptor support bits
        byte = 0;
        if let Some(descriptor) = self.static_metadata_descriptor {
            byte |= 1 << (descriptor as u8);
        }

        data.push(byte);

        self.luminance.append_to(&mut data);
        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_HDR_STATIC_METADATA_HEADER_LEN + self.luminance.size()
    }
}

#[cfg(test)]
mod test_hdr_static_metadata {
    use rstest::rstest;

    use crate::{
        EdidExtensionCTA861HdrStaticMetadataDataBlock,
        EdidExtensionCTA861HdrStaticMetadataDescriptor as Descriptor,
        EdidExtensionCTA861HdrStaticMetadataLuminance as Luminance, IntoBytes as _,
    };

    #[rstest]
    #[case::none(Luminance::None, &[0xE3, 0x06, 0x0D, 0x01])]
    #[case::max(
        Luminance::Max { max_luminance: 138 },
        &[0xE4, 0x06, 0x0D, 0x01, 0x8A],
    )]
    #[case::max_avg(
        Luminance::MaxAvg { max_luminance: 138, max_avg_luminance: 96 },
        &[0xE5, 0x06, 0x0D, 0x01, 0x8A, 0x60],
    )]
    #[case::min_max_avg(
        Luminance::MinMaxAvg {
            max_luminance: 138, max_avg_luminance: 96, min_luminance: 18,
        },
        &[0xE6, 0x06, 0x0D, 0x01, 0x8A, 0x60, 0x12],
    )]
    fn test_binary_spec(#[case] luminance: Luminance, #[case] expected: &[u8]) {
        // SDR + PQ + HLG, Type 1 static metadata, luminance code values 138/96/18.
        // Verified independently against edid-decode.
        let block = EdidExtensionCTA861HdrStaticMetadataDataBlock::builder()
            .traditional_gamma_sdr(true)
            .smpte_st2084(true)
            .bt_2100_hlg(true)
            .static_metadata_descriptor(Descriptor::Type1)
            .luminance(luminance)
            .build();

        assert_eq!(block.size(), expected.len());
        assert_eq!(block.into_bytes(), expected);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize() {
        let json = r#"{
            "static_metadata_descriptor": "type1",
            "luminance": {
                "min_max_avg": {"max_luminance":138,"max_avg_luminance":96,"min_luminance":18}
            }
        }"#;
        let block: EdidExtensionCTA861HdrStaticMetadataDataBlock =
            serde_json::from_str(json).expect("HDR block should deserialize");

        assert_eq!(
            block.into_bytes(),
            [0xE6, 0x06, 0x00, 0x01, 0x8A, 0x60, 0x12]
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_deserialize_rejects_extra_luminance() {
        let json = r#"{"luminance":{"max":{"max_luminance":138,"min_luminance":18}}}"#;

        assert!(
            serde_json::from_str::<EdidExtensionCTA861HdrStaticMetadataDataBlock>(json).is_err(),
            "Extra luminance fields should be rejected",
        );
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum EdidExtensionCTA861Revision3DataBlock {
    Audio(EdidExtensionCTA861AudioDataBlock),
    SpeakerAllocation(EdidExtensionCTA861SpeakerAllocationDataBlock),
    Colorimetry(EdidExtensionCTA861ColorimetryDataBlock),
    Video(EdidExtensionCTA861VideoDataBlock),
    #[cfg_attr(feature = "serde", serde(rename = "hdmi"))]
    HDMI(EdidExtensionCTA861HdmiDataBlock),
    VideoCapability(EdidExtensionCTA861VideoCapabilityDataBlock),
    #[cfg_attr(feature = "serde", serde(rename = "hdr"))]
    HDR(EdidExtensionCTA861HdrStaticMetadataDataBlock),
}

impl IntoBytes for EdidExtensionCTA861Revision3DataBlock {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            Self::Audio(v) => v.into_bytes(),
            Self::SpeakerAllocation(v) => v.into_bytes(),
            Self::Colorimetry(v) => v.into_bytes(),
            Self::Video(v) => v.into_bytes(),
            Self::HDMI(v) => v.into_bytes(),
            Self::VideoCapability(v) => v.into_bytes(),
            Self::HDR(v) => v.into_bytes(),
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Audio(v) => v.size(),
            Self::SpeakerAllocation(v) => v.size(),
            Self::Colorimetry(v) => v.size(),
            Self::Video(v) => v.size(),
            Self::HDMI(v) => v.size(),
            Self::VideoCapability(v) => v.size(),
            Self::HDR(v) => v.size(),
        }
    }
}

#[derive(Clone, Debug, TypedBuilder)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[builder(mutators(
    #[allow(unreachable_pub)]
    pub fn data_blocks(&mut self, blocks: Vec<EdidExtensionCTA861Revision3DataBlock>) {
        self.data_blocks = blocks;
    }

    #[allow(unreachable_pub)]
    pub fn add_data_block(&mut self, block: EdidExtensionCTA861Revision3DataBlock) {
        self.data_blocks.push(block);
    }

    #[allow(unreachable_pub)]
    pub fn detailed_timing_descriptors(&mut self, dtd: Vec<EdidDescriptorDetailedTiming>) {
        self.timings = dtd;
    }

    #[allow(unreachable_pub)]
    pub fn add_detailed_timing_descriptor(&mut self, dtd: EdidDescriptorDetailedTiming) {
        self.timings.push(dtd);
    }
))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct EdidExtensionCTA861Revision3 {
    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    ycbcr_422_supported: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    ycbcr_444_supported: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    audio_supported: bool,

    #[builder(default)]
    #[cfg_attr(feature = "serde", serde(default))]
    underscan_it_formats_by_default: bool,

    native_formats: u8,

    #[builder(via_mutators)]
    #[cfg_attr(feature = "serde", serde(default))]
    data_blocks: Vec<EdidExtensionCTA861Revision3DataBlock>,

    #[builder(via_mutators)]
    #[cfg_attr(feature = "serde", serde(default))]
    timings: Vec<EdidDescriptorDetailedTiming>,
}

impl IntoBytes for EdidExtensionCTA861Revision3 {
    fn into_bytes(self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(EDID_EXTENSION_CTA_861_LEN);

        data.extend_from_slice(&[0x02, 0x03]);

        let dtd_offset = if self.data_blocks.is_empty() && self.timings.is_empty() {
            0
        } else {
            self.data_blocks
                .iter()
                .fold(4, |acc, b| acc + b.size())
                .to_u8()
                .expect("The number of data blocks would overflow our type")
        };
        data.push(dtd_offset);

        let mut byte = 0;
        if self.underscan_it_formats_by_default {
            byte |= 1 << 7;
        }

        if self.audio_supported {
            byte |= 1 << 6;
        }

        if self.ycbcr_444_supported {
            byte |= 1 << 5;
        }

        if self.ycbcr_422_supported {
            byte |= 1 << 4;
        }

        byte |= self.native_formats;
        data.push(byte);

        for block in self.data_blocks {
            data.extend_from_slice(&block.into_bytes());
        }

        for timing in self.timings {
            data.extend_from_slice(&timing.into_bytes());
        }

        data.resize(EDID_EXTENSION_CTA_861_LEN - 1, 0);

        let mut sum: u8 = 0;
        for byte in &data {
            sum = sum.wrapping_add(*byte);
        }

        let checksum = 0u8.wrapping_sub(sum);
        data.push(checksum);

        assert_eq!(
            data.len(),
            EDID_EXTENSION_CTA_861_LEN,
            "EDID CTA-861 Extension is larger than it should ({} vs expected {} bytes)",
            data.len(),
            EDID_EXTENSION_CTA_861_LEN
        );

        data
    }

    fn size(&self) -> usize {
        EDID_EXTENSION_CTA_861_LEN
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum EdidExtensionCTA861 {
    Revision3(EdidExtensionCTA861Revision3),
}

impl IntoBytes for EdidExtensionCTA861 {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            EdidExtensionCTA861::Revision3(v) => v.into_bytes(),
        }
    }

    fn size(&self) -> usize {
        match self {
            EdidExtensionCTA861::Revision3(v) => v.size(),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum EdidExtension {
    CTA861(EdidExtensionCTA861),
}

impl IntoBytes for EdidExtension {
    fn into_bytes(self) -> Vec<u8> {
        match self {
            EdidExtension::CTA861(v) => v.into_bytes(),
        }
    }

    fn size(&self) -> usize {
        match self {
            EdidExtension::CTA861(v) => v.size(),
        }
    }
}
