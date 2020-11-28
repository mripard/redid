#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

use std::io::Write;
use enum_map::Enum;
use enum_map::EnumMap;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDVersion {
    V1R4,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDWeekYear {
    YearOfManufacture(u16),
    WeekYearOfManufacture(u8, u16),
    ModelYear(u16),
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDVideoDigitalColorDepth {
    Undefined = 0,
    Depth6bpc,
    Depth8bpc,
    Depth10bpc,
    Depth12bpc,
    Depth14bpc,
    Depth16bpc,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDVideoDigitalInterfaceStandard {
    Undefined = 0,
    DVI,
    HDMIa,
    HDMIb,
    MDDI,
    DisplayPort,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct EDIDVideoDigitalInterface {
    color_depth: EDIDVideoDigitalColorDepth,
    interface: EDIDVideoDigitalInterfaceStandard,
}

impl EDIDVideoDigitalInterface {
    pub fn new(interface: EDIDVideoDigitalInterfaceStandard, bpc: EDIDVideoDigitalColorDepth) -> Self {
        EDIDVideoDigitalInterface {
            interface,
            color_depth: bpc,
        }
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDVideoInput {
    Digital(EDIDVideoDigitalInterface),
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDScreenSizeRatio {
    None,
    Size(u8, u8),
    LandscapeRatio(f32),
    PortraitRatio(f32),
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDisplayColorType {
    MonochromeGrayScale,
    RGBColor,
    NonRGBColor,
    Undefined,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDisplayColorEncoding {
    RGB444,
    RGB444YCbCr444,
    RGB444YCbCr422,
    RGB444YCbCr444YCbCr422,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDisplayColorTypeEncoding {
    ColorEncoding(EDIDDisplayColorEncoding)
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Default)]
pub struct EDIDChromaPoint {
    x: u16,
    y: u16,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Enum)]
pub enum EDIDChromaCoordinate {
    White,
    Blue,
    Red,
    Green,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDEstablishedTiming {
    ET_1024_768_60Hz,
    ET_1024_768_70Hz,
    ET_1024_768_75Hz,
    ET_1024_768_87Hz_Interlaced,
    ET_1152_870_75Hz,
    ET_1280_1024_75Hz,
    ET_640_480_60Hz,
    ET_640_480_67Hz,
    ET_640_480_72Hz,
    ET_640_480_75Hz,
    ET_720_400_70Hz,
    ET_720_400_88Hz,
    ET_800_600_56Hz,
    ET_800_600_60Hz,
    ET_800_600_72Hz,
    ET_800_600_75Hz,
    ET_832_624_75Hz,
}

#[allow(non_camel_case_types)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDStandardTimingRatio {
    Ratio_16_10,
    Ratio_4_3,
    Ratio_5_4,
    Ratio_16_9,
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub struct EDIDStandardTiming {
    x: u16,
    ratio: EDIDStandardTimingRatio,
    frequency: u8,
}

impl EDIDStandardTiming {
    pub fn new(x: u16, ratio: EDIDStandardTimingRatio, frequency: u8) -> Self {
        EDIDStandardTiming {
            x,
            ratio,
            frequency,
        }
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDetailedTimingAnalogSync {
    BipolarComposite(bool, bool),
    Composite(bool, bool),
}

impl Default for EDIDDetailedTimingAnalogSync {
        fn default() -> Self {
            EDIDDetailedTimingAnalogSync::Composite(false, false)
        }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDetailedTimingDigitalSync {
    Separate(bool, bool),
}

impl Default for EDIDDetailedTimingDigitalSync {
        fn default() -> Self {
            EDIDDetailedTimingDigitalSync::Separate(false, false)
        }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDetailedTimingSync {
    Analog(EDIDDetailedTimingAnalogSync),
    Digital(EDIDDetailedTimingDigitalSync),
}

impl Default for EDIDDetailedTimingSync {
    fn default() -> Self {
        EDIDDetailedTimingSync::Digital(EDIDDetailedTimingDigitalSync::default())
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDetailedTimingStereo {
    None,
    FieldSequentialRightOnSync,
    FieldSequentialLeftOnSync,
    TwoWayInterleavedRightOnEven,
    TwoWayInterleavedLeftOnEven,
    FourWayInterleaved,
    SideBySideInterleaved,
}

impl Default for EDIDDetailedTimingStereo {
    fn default() -> Self {
        EDIDDetailedTimingStereo::None
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Default)]
pub struct EDIDDetailedTiming {
    interlace: bool,
    pixel_clock: u32,
    sync_type: EDIDDetailedTimingSync,
    stereo: EDIDDetailedTimingStereo,

    hsize: u16,
    vsize: u16,

    hfp: u16,
    hdisplay: u16,
    hblank: u16,
    hsync: u16,

    vfp: u16,
    vdisplay: u16,
    vblank: u16,
    vsync: u16,
}

impl EDIDDetailedTiming {
    pub fn new() -> Self {
        EDIDDetailedTiming::default()
    }

    pub fn set_blanking(mut self, h:u16, v: u16) -> Self {
        self.hblank = h;
        self.vblank = v;
        self
    }

    pub fn set_display(mut self, h:u16, v: u16) -> Self {
        self.hdisplay = h;
        self.vdisplay = v;
        self
    }

    pub fn set_front_porch(mut self, h:u16, v: u16) -> Self {
        self.hfp = h;
        self.vfp = v;
        self
    }

    pub fn set_sync_pulse(mut self, h:u16, v: u16) -> Self {
        self.hsync = h;
        self.vsync = v;
        self
    }

    pub fn set_sync_type(mut self, sync: EDIDDetailedTimingSync) -> Self {
        self.sync_type = sync;
        self
    }

    pub fn set_interlace(mut self, interlace: bool) -> Self {
        self.interlace = interlace;
        self
    }

    pub fn set_pixel_clock(mut self, pc: u32) -> Self {
        self.pixel_clock = pc;
        self
    }

    pub fn set_stereo(mut self, stereo: EDIDDetailedTimingStereo) -> Self {
        self.stereo = stereo;
        self
    }

    pub fn set_size(mut self, h: u16, v: u16) -> Self {
        self.hsize = h;
        self.vsize = v;
        self
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDDisplayRangeLimitsSubtype {
    DefaultGTF,
    RangeLimitsOnly,
}

impl Default for EDIDDisplayRangeLimitsSubtype {
    fn default() -> Self {
        EDIDDisplayRangeLimitsSubtype::DefaultGTF
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(Default)]
pub struct EDIDDisplayRangeLimits {
    min_hfreq: u16,
    max_hfreq: u16,
    min_vfreq: u16,
    max_vfreq: u16,
    max_pixelclock: u16,

    subtype: EDIDDisplayRangeLimitsSubtype,
}

impl EDIDDisplayRangeLimits {
    pub fn new() -> Self {
        EDIDDisplayRangeLimits::default()
    }

    pub fn set_subtype(mut self, subtype: EDIDDisplayRangeLimitsSubtype) -> Self {
        self.subtype = subtype;
        self
    }

    pub fn set_pixel_clock_max(mut self, max: u16) -> Self {
        self.max_pixelclock = max;
        self
    }

    pub fn set_horizontal_rate_range(mut self, min: u16, max: u16) -> Self {
        self.min_hfreq = min;
        self.max_hfreq = max;
        self
    }

    pub fn set_vertical_rate_range(mut self, min: u16, max: u16) -> Self {
        self.min_vfreq = min;
        self.max_vfreq = max;
        self
    }
}

#[derive(Clone)]
#[derive(Debug)]
pub enum EDIDDescriptor {
    Custom(u8, Vec<u8>),
    Dummy,
    DataString(String),
    ProductName(String),
    ProductSerialNumber(String),
    DetailedTiming(EDIDDetailedTiming),
    DisplayRangeLimits(EDIDDisplayRangeLimits),
}

#[derive(Debug)]
pub struct EDID {
    // EDID Version
    version: EDIDVersion,

    // Vendor and Product identification
    manufacturer: [u8; 3],
    product: u16,
    serial: u32,
    week_year: EDIDWeekYear,

    // Basic Display Parameters
    input: EDIDVideoInput,
    size_ratio: EDIDScreenSizeRatio,
    gamma: f32,
    feature_standby: bool,
    feature_suspend: bool,
    feature_active_off: bool,
    feature_continuous_frequency: bool,
    feature_color_type_encoding: EDIDDisplayColorTypeEncoding,
    feature_srgb_default: bool,
    feature_preferred_timings_native: bool,

    chroma_coord: EnumMap<EDIDChromaCoordinate, EDIDChromaPoint>,
    established_timings: Vec<EDIDEstablishedTiming>,
    standard_timings: Vec<EDIDStandardTiming>,
    descriptors: Vec<EDIDDescriptor>,
}

impl EDID {
    pub fn new(version: EDIDVersion) -> Self {
        EDID {
            version,
            manufacturer: ['R' as u8, 'S' as u8, 'T' as u8],
            product: 0,
            serial: 0,
            week_year: EDIDWeekYear::ModelYear(1990),
            input: EDIDVideoInput::Digital(EDIDVideoDigitalInterface {
                color_depth: EDIDVideoDigitalColorDepth::Undefined,
                interface: EDIDVideoDigitalInterfaceStandard::Undefined,
            }),
            size_ratio: EDIDScreenSizeRatio::None,
            gamma: 2.20,
            feature_standby: false,
            feature_suspend: false,
            feature_active_off: false,
            feature_continuous_frequency: false,
            feature_color_type_encoding: EDIDDisplayColorTypeEncoding::ColorEncoding(EDIDDisplayColorEncoding::RGB444),
            feature_srgb_default: false,
            feature_preferred_timings_native: false,

            chroma_coord: EnumMap::<EDIDChromaCoordinate, EDIDChromaPoint>::new(),
            established_timings: Vec::new(),
            standard_timings: Vec::new(),
            descriptors: Vec::new(),
        }
    }

    pub fn add_established_timing(mut self, et: EDIDEstablishedTiming) -> Self {
        self.established_timings.push(et);
        self
    }

    pub fn add_standard_timing(mut self, st: EDIDStandardTiming) -> Self {
        self.standard_timings.push(st);
        self
    }

    pub fn add_descriptor(mut self, desc: EDIDDescriptor) -> Self {
        self.descriptors.push(desc);
        self
    }

    pub fn set_continuous_frequency(mut self, cf: bool) -> Self {
        self.feature_continuous_frequency = cf;
        self
    }

    pub fn set_display_color_type_encoding(mut self, color: EDIDDisplayColorTypeEncoding) -> Self {
        self.feature_color_type_encoding = color;
        self
    }

    pub fn set_input(mut self, input: EDIDVideoInput) -> Self {
        self.input = input;
        self
    }

    pub fn set_dpm_active_off(mut self, active_off: bool) -> Self {
        self.feature_active_off = active_off;
        self
    }

    pub fn set_dpm_standby(mut self, standby: bool) -> Self {
        self.feature_standby = standby;
        self
    }

    pub fn set_dpm_suspend(mut self, suspend: bool) -> Self {
        self.feature_suspend = suspend;
        self
    }

    pub fn set_gamma(mut self, gamma: f32) -> Self {
        self.gamma = gamma;
        self
    }

    pub fn set_screen_size_ratio(mut self, ratio: EDIDScreenSizeRatio) -> Self {
        self.size_ratio = ratio;
        self
    }

    pub fn set_preferred_timings_native(mut self, native: bool) -> Self {
        self.feature_preferred_timings_native = native;
        self
    }

    pub fn set_srgb_default(mut self, default: bool) -> Self {
        self.feature_srgb_default = default;
        self
    }

    pub fn set_serial_number(mut self, serial: u32) -> Self {
        self.serial = serial;
        self
    }

    pub fn set_week_year(mut self, date: EDIDWeekYear) -> Self {
        self.week_year = date;
        self
    }

    pub fn set_manufacturer_id(mut self, id: &str) -> Self {
        self.manufacturer.copy_from_slice(&id.as_bytes()[0..3]);
        self
    }

    pub fn set_product_id(mut self, id: u16) -> Self {
        self.product = id;
        self
    }

    pub fn set_chroma_coordinates(mut self, chroma: EDIDChromaCoordinate, x: u16, y: u16) -> Self {
        self.chroma_coord[chroma].x = x;
        self.chroma_coord[chroma].y = y;
        self
    }

    pub fn serialize(self, writer: &mut impl Write) {
        writer.write(&[0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00]).unwrap();

        let manufacturer = &self.manufacturer;
        let mut comp = ((manufacturer[0] as u8) - ('@' as u8)) << 2;
        comp |= ((manufacturer[1] as u8) - ('@' as u8)) >> 3;
        writer.write(&[comp]).unwrap();

        comp = ((manufacturer[1] as u8) - ('@' as u8)) << 5;
        comp |= (manufacturer[2] as u8) - ('@' as u8);
        writer.write(&[comp]).unwrap();

        let prod = &self.product;
        writer.write(&[(prod & 0xff) as u8]).unwrap();
        writer.write(&[(prod >> 8) as u8]).unwrap();

        let serial = &self.serial;
        writer.write(&[(serial & 0xff) as u8]).unwrap();
        writer.write(&[((serial >> 8) & 0xff) as u8]).unwrap();
        writer.write(&[((serial >> 16) & 0xff) as u8]).unwrap();
        writer.write(&[((serial >> 24) & 0xff) as u8]).unwrap();

        match self.week_year {
            EDIDWeekYear::ModelYear(year) => {
                writer.write(&[0xff]).unwrap();
                writer.write(&[(year - 1990) as u8]).unwrap();
            },
            EDIDWeekYear::YearOfManufacture(year) => {
                writer.write(&[0x00]).unwrap();
                writer.write(&[(year - 1990) as u8]).unwrap();
            },
            EDIDWeekYear::WeekYearOfManufacture(week, year) => {
                writer.write(&[week]).unwrap();
                writer.write(&[(year - 1990) as u8]).unwrap();
            }
        }

        match self.version {
            EDIDVersion::V1R4 => {
                writer.write(&[1]).unwrap();
                writer.write(&[4]).unwrap();
            },
        }

        match self.input {
            EDIDVideoInput::Digital(itf) => {
                let mut byte: u8 = 0x80;

                byte |= (itf.color_depth as u8) << 4;
                byte |= itf.interface as u8;
                writer.write(&[byte]).unwrap();
            }
        }

        match self.size_ratio {
            EDIDScreenSizeRatio::LandscapeRatio(ratio) => {
                let stored = (ratio * 100.0 - 99.0).round() as u8;
                writer.write(&[stored]).unwrap();
                writer.write(&[0]).unwrap();
            },
            EDIDScreenSizeRatio::PortraitRatio(ratio) => {
                let stored = (100.0 / ratio - 99.0).round() as u8;
                writer.write(&[0]).unwrap();
                writer.write(&[stored]).unwrap();
            },
            EDIDScreenSizeRatio::Size(x, y) => {
                writer.write(&[x]).unwrap();
                writer.write(&[y]).unwrap();
            },
            EDIDScreenSizeRatio::None => {
                writer.write(&[0]).unwrap();
                writer.write(&[0]).unwrap();
            }
        }

        let gamma = (self.gamma * 100.0 - 100.0).round() as u8;
        writer.write(&[gamma]).unwrap();

        let mut feature: u8 = 0;
        if self.feature_standby {
            feature |= 1 << 7;
        }

        if self.feature_suspend {
            feature |= 1 << 6;
        }

        if self.feature_active_off {
            feature |= 1 << 5;
        }

        if self.feature_srgb_default {
            feature |= 1 << 2;
        }

        if self.feature_preferred_timings_native {
            feature |= 1 << 1;
        }

        if self.feature_continuous_frequency {
            feature |= 1;
        }

        // FIXME: Support Color Type for Analog
        match self.input {
            EDIDVideoInput::Digital(_) => {
                match self.feature_color_type_encoding {
                    EDIDDisplayColorTypeEncoding::ColorEncoding(enc) => {
                        feature |= (enc as u8) << 3;
                    },
                }
            }
        }

        // FIXME: Other features support
        writer.write(&[feature]).unwrap();

        // Chromaticity Coordinates
        let blue_x = self.chroma_coord[EDIDChromaCoordinate::Blue].x;
        let blue_y = self.chroma_coord[EDIDChromaCoordinate::Blue].y;
        let red_x = self.chroma_coord[EDIDChromaCoordinate::Red].x;
        let red_y = self.chroma_coord[EDIDChromaCoordinate::Red].y;
        let green_x = self.chroma_coord[EDIDChromaCoordinate::Green].x;
        let green_y = self.chroma_coord[EDIDChromaCoordinate::Green].y;
        let white_x = self.chroma_coord[EDIDChromaCoordinate::White].x;
        let white_y = self.chroma_coord[EDIDChromaCoordinate::White].y;

        let mut byte: u8 = ((red_x & 0b11) << 6) as u8;
        byte |= ((red_y & 0b11) << 4) as u8;
        byte |= ((green_x & 0b11) << 2) as u8;
        byte |= (green_y & 0b11) as u8;
        writer.write(&[byte]).unwrap();

        byte = ((blue_x & 0b11) << 6) as u8;
        byte |= ((blue_y & 0b11) << 4) as u8;
        byte |= ((white_x & 0b11) << 2) as u8;
        byte |= (white_y & 0b11) as u8;
        writer.write(&[byte]).unwrap();

        writer.write(&[(red_x >> 2) as u8]).unwrap();
        writer.write(&[(red_y >> 2) as u8]).unwrap();
        writer.write(&[(green_x >> 2) as u8]).unwrap();
        writer.write(&[(green_y >> 2) as u8]).unwrap();
        writer.write(&[(blue_x >> 2) as u8]).unwrap();
        writer.write(&[(blue_y >> 2) as u8]).unwrap();
        writer.write(&[(white_x >> 2) as u8]).unwrap();
        writer.write(&[(white_y >> 2) as u8]).unwrap();

        let mut byte0: u8 = 0;
        let mut byte1: u8 = 0;
        let mut byte2: u8 = 0;
        for et in self.established_timings {
            match et {
                EDIDEstablishedTiming::ET_800_600_60Hz => byte0 |= (1 << 0) as u8,
                EDIDEstablishedTiming::ET_800_600_56Hz => byte0 |= (1 << 1) as u8,
                EDIDEstablishedTiming::ET_640_480_75Hz => byte0 |= (1 << 2) as u8,
                EDIDEstablishedTiming::ET_640_480_72Hz => byte0 |= (1 << 3) as u8,
                EDIDEstablishedTiming::ET_640_480_67Hz => byte0 |= (1 << 4) as u8,
                EDIDEstablishedTiming::ET_640_480_60Hz => byte0 |= (1 << 5) as u8,
                EDIDEstablishedTiming::ET_720_400_88Hz => byte0 |= (1 << 6) as u8,
                EDIDEstablishedTiming::ET_720_400_70Hz => byte0 |= (1 << 7) as u8,
                EDIDEstablishedTiming::ET_1280_1024_75Hz => byte1 |= (1 << 0) as u8,
                EDIDEstablishedTiming::ET_1024_768_75Hz => byte1 |= (1 << 1) as u8,
                EDIDEstablishedTiming::ET_1024_768_70Hz => byte1 |= (1 << 2) as u8,
                EDIDEstablishedTiming::ET_1024_768_60Hz => byte1 |= (1 << 3) as u8,
                EDIDEstablishedTiming::ET_1024_768_87Hz_Interlaced => byte1 |= (1 << 4) as u8,
                EDIDEstablishedTiming::ET_832_624_75Hz => byte1 |= (1 << 5) as u8,
                EDIDEstablishedTiming::ET_800_600_75Hz => byte1 |= (1 << 6) as u8,
                EDIDEstablishedTiming::ET_800_600_72Hz => byte1 |= (1 << 7) as u8,
                EDIDEstablishedTiming::ET_1152_870_75Hz => byte2 |= (1 << 7) as u8,
            };
        }
        writer.write(&[byte0, byte1, byte2]).unwrap();

        for st_idx in 0..8 {
            let st = self.standard_timings.get(st_idx);
            match st {
                Some(timing) => {
                    let byte0 = ((timing.x / 8) - 31) as u8;

                    let mut byte1 = (timing.frequency - 60) & 0x3f as u8;
                    let ratio: u8 = match timing.ratio {
                        EDIDStandardTimingRatio::Ratio_16_10 => 0,
                        EDIDStandardTimingRatio::Ratio_4_3 => 1,
                        EDIDStandardTimingRatio::Ratio_5_4 => 2,
                        EDIDStandardTimingRatio::Ratio_16_9 => 3,
                    };
                    byte1 |= ratio << 6;

                    writer.write(&[byte0, byte1])
                },
                None => {
                    writer.write(&[1, 1])
                },
            }.unwrap();
        }

        for desc_idx in 0..4 {
            let desc = self.descriptors.get(desc_idx);
            match desc {
                Some(desc_type) => {
                    match desc_type {
                        EDIDDescriptor::Custom(tag, val) => {
                            let mut data: Vec<u8> = Vec::with_capacity(18);

                            data.extend_from_slice(&[0, 0, 0, *tag, 0]);

                            let len = val.len();
                            if len >= 13 {
                                data.extend_from_slice(&val[0..13]);
                            } else {
                                data.extend_from_slice(val);

                                for _ in 0..(13 - len) {
                                    data.push(0);
                                }
                            }

                            writer.write(&data)
                        },
                        EDIDDescriptor::DetailedTiming(dt) => {
                            let freq = (dt.pixel_clock / 10) as u16;
                            let lo_freq = (freq & 0xff) as u8;
                            let hi_freq = ((freq >> 8) & 0xff) as u8;
                            let lo_haddr = (dt.hdisplay & 0xff) as u8;
                            let hi_haddr = ((dt.hdisplay >> 8) & 0xf) as u8;
                            let lo_hblank = (dt.hblank & 0xff) as u8;
                            let hi_hblank = ((dt.hblank >> 8) & 0xf) as u8;
                            let lo_vaddr = (dt.vdisplay & 0xff) as u8;
                            let hi_vaddr = ((dt.vdisplay >> 8) & 0xf) as u8;
                            let lo_vblank = (dt.vblank & 0xff) as u8;
                            let hi_vblank = ((dt.vblank >> 8) & 0xf) as u8;
                            let lo_hfp = (dt.hfp & 0xff) as u8;
                            let hi_hfp = ((dt.hfp >> 8) & 0x3) as u8;
                            let lo_hsync = (dt.hsync & 0xff) as u8;
                            let hi_hsync = ((dt.hsync >> 8) & 0x3) as u8;
                            let lo_vfp = (dt.vfp & 0xf) as u8;
                            let hi_vfp = ((dt.vfp >> 4) & 0x3) as u8;
                            let lo_vsync = (dt.vsync & 0xf) as u8;
                            let hi_vsync = ((dt.vsync >> 4) & 0x3) as u8;
                            let lo_hsize = (dt.hsize & 0xff) as u8;
                            let hi_hsize = ((dt.hsize >> 8) & 0xf) as u8;
                            let lo_vsize = (dt.vsize & 0xff) as u8;
                            let hi_vsize = ((dt.vsize >> 8) & 0xf) as u8;

                            let mut flags: u8 = 0;

                            if dt.interlace {
                                flags |= 1 << 7;
                            }

                            match dt.stereo {
                                EDIDDetailedTimingStereo::None => flags |= 0,
                                EDIDDetailedTimingStereo::FieldSequentialRightOnSync => flags |= 0b0100000,
                                EDIDDetailedTimingStereo::FieldSequentialLeftOnSync => flags |= 0b1000000,
                                EDIDDetailedTimingStereo::TwoWayInterleavedRightOnEven => flags |= 0b0100001,
                                EDIDDetailedTimingStereo::TwoWayInterleavedLeftOnEven => flags |= 0b1000001,
                                EDIDDetailedTimingStereo::FourWayInterleaved => flags |= 0b1100000,
                                EDIDDetailedTimingStereo::SideBySideInterleaved => flags |= 0b1100001,
                            }

                            match dt.sync_type {
                                EDIDDetailedTimingSync::Analog(sync) => {
                                    match sync {
                                        EDIDDetailedTimingAnalogSync::BipolarComposite(serrations, sync_on_rgb) => {
                                            flags |= 1 << 3;

                                            if serrations {
                                                flags |= 1 << 2;
                                            }

                                            if sync_on_rgb {
                                                flags |= 1 << 1;
                                            }
                                        },
                                        EDIDDetailedTimingAnalogSync::Composite(serrations, sync_on_rgb) => {
                                            if serrations {
                                                flags |= 1 << 2;
                                            }

                                            if sync_on_rgb {
                                                flags |= 1 << 1;
                                            }
                                        },
                                    }
                                },
                                EDIDDetailedTimingSync::Digital(sync) => {
                                    match sync {
                                        EDIDDetailedTimingDigitalSync::Separate(hpol, vpol) => {
                                            flags |= 0b11000;

                                            if vpol {
                                                flags |= 1 << 2;
                                            }

                                            if hpol {
                                                flags |= 1 << 1;
                                            }
                                        },
                                    };
                                },
                            }

                            writer.write(&[lo_freq, hi_freq,
                                lo_haddr, lo_hblank, (hi_haddr << 4) | hi_hblank,
                                lo_vaddr, lo_vblank, (hi_vaddr << 4) | hi_vblank,
                                lo_hfp, lo_hsync,
                                (lo_vfp << 4) | lo_vsync,
                                (hi_hfp << 6) | (hi_hsync << 4) | (hi_vfp << 2) | hi_vsync,
                                lo_hsize, lo_vsize, (hi_hsize << 4) | hi_vsize,
                                // FIXME: Borders size
                                0, 0,
                                flags])
                        },
                        EDIDDescriptor::DisplayRangeLimits(limits) => {
                            let mut data: Vec<u8> = Vec::with_capacity(18);
                            let mut flags_byte: u8 = 0;

                            data.extend_from_slice(&[0, 0, 0, 0xfd]);

                            let mut max_hfreq = limits.max_hfreq;
                            if max_hfreq > 255 {
                                flags_byte |= 1 << 3;
                                max_hfreq = max_hfreq - 255;
                            }

                            let mut min_hfreq = limits.min_hfreq;
                            if min_hfreq > 255 {
                                flags_byte |= 1 << 2;
                                min_hfreq = min_hfreq - 255;
                            }

                            let mut max_vfreq = limits.max_vfreq;
                            if max_vfreq > 255 {
                                flags_byte |= 1 << 1;
                                max_vfreq = max_vfreq - 255;
                            }

                            let mut min_vfreq = limits.min_vfreq;
                            if min_vfreq > 255 {
                                flags_byte |= 1;
                                min_vfreq = min_vfreq - 255;
                            }
                            data.push(flags_byte);

                            data.push(min_vfreq as u8);
                            data.push(max_vfreq as u8);
                            data.push(min_hfreq as u8);
                            data.push(max_hfreq as u8);
                            data.push((limits.max_pixelclock / 10) as u8);
                            data.push(match limits.subtype {
                                EDIDDisplayRangeLimitsSubtype::DefaultGTF => 0,
                                EDIDDisplayRangeLimitsSubtype::RangeLimitsOnly => 1,
                            } as u8);
                            data.push(0x0a);
                            data.extend_from_slice(&[0x20, 0x20, 0x20, 0x20, 0x20, 0x20]);

                            writer.write(&data)
                        },
                        EDIDDescriptor::DataString(string) => {
                            let mut data: Vec<u8> = Vec::with_capacity(18);

                            data.extend_from_slice(&[0, 0, 0, 0xfe, 0]);

                            let bytes = string.as_bytes();
                            let mut count = 0;
                            for byte in bytes {
                                if *byte == 0xc2 {
                                    continue
                                }

                                data.push(*byte);
                                count = count + 1;
                            }

                            if count < 13 {
                                data.push(0x0a);
                                count = count +  1;
                            }

                            for _ in count..13 {
                                data.push(0x20);
                            }

                            writer.write(&data)
                        },
                        EDIDDescriptor::Dummy => {
                            writer.write(&[0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
                        },
                        EDIDDescriptor::ProductName(name) => {
                            let mut data: Vec<u8> = Vec::with_capacity(18);

                            data.extend_from_slice(&[0, 0, 0, 0xfc, 0]);

                            let len = name.len();
                            if len >= 13 {
                                data.extend_from_slice(&name.as_bytes()[0..13]);
                            } else {
                                data.extend_from_slice(&name.as_bytes()[0..len]);
                                data.push(0x0a);

                                for _ in 0..(13 - len - 1) {
                                    data.push(0x20);
                                }
                            }

                            writer.write(&data)
                        },
                        EDIDDescriptor::ProductSerialNumber(serial) => {
                            let mut data: Vec<u8> = Vec::with_capacity(18);

                            data.extend_from_slice(&[0, 0, 0, 0xff, 0]);

                            let len = serial.len();
                            if len >= 13 {
                                data.extend_from_slice(&serial.as_bytes()[0..13]);
                            } else {
                                data.extend_from_slice(&serial.as_bytes()[0..len]);
                                data.push(0x0a);

                                for _ in 0..(13 - len - 1) {
                                    data.push(0x20);
                                }
                            }

                            writer.write(&data)
                        },
                    }
                },
                None => writer.write(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            }.unwrap();
        }

        // FIXME: Support the extensions
        writer.write(&[0]).unwrap();

        // FIXME: Support the cheksum
        writer.write(&[0]).unwrap();
    }
}
