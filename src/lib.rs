#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

use std::io::Write;

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum EDIDVersion {
    V1R4,
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
    Size(u8, u8),
    LandscapeRatio(u8, u8),
    PortraitRatio(u8, u8),
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

#[derive(Debug)]
pub struct EDID {
    // EDID Version
    version: EDIDVersion,

    // Vendor and Product identification
    manufacturer: [u8; 3],
    product: u16,
    serial: u32,
    week: u8,
    year: u16,

    // Basic Display Parameters
    input: EDIDVideoInput,
    size_ratio: EDIDScreenSizeRatio,
    gamma: f32,
    feature_standby: bool,
    feature_suspend: bool,
    feature_active_off: bool,
    feature_color_type_encoding: EDIDDisplayColorTypeEncoding,

    // FIXME: Add Color Charateristics
}

impl EDID {
    pub fn new(version: EDIDVersion) -> Self {
        EDID {
            version,
            manufacturer: ['R' as u8, 'S' as u8, 'T' as u8],
            product: 0,
            serial: 0,
            week: 1,
            year: 2020,
            input: EDIDVideoInput::Digital(EDIDVideoDigitalInterface {
                color_depth: EDIDVideoDigitalColorDepth::Undefined,
                interface: EDIDVideoDigitalInterfaceStandard::Undefined,
            }),
            size_ratio: EDIDScreenSizeRatio::LandscapeRatio(4, 3),
            gamma: 2.20,
            feature_standby: false,
            feature_suspend: false,
            feature_active_off: false,
            feature_color_type_encoding: EDIDDisplayColorTypeEncoding::ColorEncoding(EDIDDisplayColorEncoding::RGB444),
        }
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

        // FIXME: Allow a model year as well
        writer.write(&[self.week]).unwrap();
        writer.write(&[(self.year - 1990) as u8]).unwrap();

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
            EDIDScreenSizeRatio::LandscapeRatio(num, denum) => {
                let ratio =  num as f32 / denum as f32;
                let stored = (ratio * 100.0 - 99.0).round() as u8;
                writer.write(&[stored]).unwrap();
                writer.write(&[0]).unwrap();
            },
            EDIDScreenSizeRatio::PortraitRatio(num, denum) => {
                let ratio =  num as f32 / denum as f32;
                let stored = (100.0 / ratio - 99.0).round() as u8;
                writer.write(&[0]).unwrap();
                writer.write(&[stored]).unwrap();
            },
            EDIDScreenSizeRatio::Size(x, y) => {
                writer.write(&[x]).unwrap();
                writer.write(&[y]).unwrap();
            },
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

        // FIXME: Support Display Chromaticity Coordinates 
        writer.write(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();

        // FIXME: Support the Established Timings
        writer.write(&[0, 0, 0]).unwrap();

        // FIXME: Support the Standard Timings
        writer.write(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();

        // FIXME: Support the Descriptors
        for _ in 0..4 {
            writer.write(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        }

        // FIXME: Support the extensions
        writer.write(&[0]).unwrap();

        // FIXME: Support the cheksum
        writer.write(&[0]).unwrap();
    }
}
