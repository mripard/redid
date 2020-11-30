fn round_up(number: u16, multiple: u16) -> u16 {
    if (number % multiple) == 0 {
        return number;
    }

    ((number / multiple) + 1) * 10
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
    max_pixelclock: u32,

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

    pub fn set_pixel_clock_max(mut self, max: u32) -> Self {
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

impl EDIDDescriptor {
    pub(crate) fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(18);

        match self {
            EDIDDescriptor::Custom(tag, val) => {
                data.extend_from_slice(&[0, 0, 0, *tag, 0]);

                let len = val.len();
                if len >= 13 {
                    data.extend_from_slice(&val[0..13]);
                } else {
                    data.extend_from_slice(&val);

                    for _ in 0..(13 - len) {
                        data.push(0);
                    }
                }
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

                data.extend_from_slice(&[lo_freq, hi_freq,
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

                let pclk = limits.max_pixelclock;
                let pclk_mhz = (pclk / 1000) as u16;
                let rounded_pclk_mhz = round_up(pclk_mhz, 10);
                data.push((rounded_pclk_mhz / 10) as u8);
                data.push(match limits.subtype {
                    EDIDDisplayRangeLimitsSubtype::DefaultGTF => 0,
                    EDIDDisplayRangeLimitsSubtype::RangeLimitsOnly => 1,
                } as u8);
                data.push(0x0a);
                data.extend_from_slice(&[0x20, 0x20, 0x20, 0x20, 0x20, 0x20]);
            },
            EDIDDescriptor::DataString(string) => {
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
            },
            EDIDDescriptor::Dummy => {
                data.extend_from_slice(&[0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0])
            },
            EDIDDescriptor::ProductName(name) => {
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
            },
            EDIDDescriptor::ProductSerialNumber(serial) => {
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
            },
        }

        data
    }
}