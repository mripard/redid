extern crate serde_json;
extern crate test_generator;

use std::fs::File;
use std::io::Read;
use std::process::Command;

use edid::EDIDChromaCoordinate;
use edid::EDIDDescriptor;
use edid::EDIDDetailedTiming;
use edid::EDIDDetailedTimingAnalogSync;
use edid::EDIDDetailedTimingDigitalSync;
use edid::EDIDDetailedTimingStereo;
use edid::EDIDDetailedTimingSync;
use edid::EDIDDisplayColorEncoding;
use edid::EDIDDisplayColorTypeEncoding;
use edid::EDIDDisplayRangeLimits;
use edid::EDIDDisplayRangeLimitsSubtype;
use edid::EDIDEstablishedTiming;
use edid::EDIDScreenSizeRatio;
use edid::EDIDStandardTiming;
use edid::EDIDStandardTimingRatio;
use edid::EDIDVersion;
use edid::EDIDVideoDigitalColorDepth;
use edid::EDIDVideoDigitalInterface;
use edid::EDIDVideoDigitalInterfaceStandard;
use edid::EDIDVideoInput;
use edid::EDIDWeekYear;
use edid::EDID;

use serde_json::Value;
use test_generator::test_resources;

fn decode_manufacturer_info(edid: &mut EDID, manufacturer_info: &Value) {
    let manufacturer_id_val = manufacturer_info["Manufacturer ID"]
        .as_str()
        .expect("Couldn't decode the manufacturer ID");
    edid.set_manufacturer_id(manufacturer_id_val);

    let product_id_val = manufacturer_info["ID Product Code"]
        .as_u64()
        .expect("Couldn't decode the product ID") as u16;
    edid.set_product_id(product_id_val);

    manufacturer_info["Serial number"]
        .as_u64()
        .map(|serial| edid.set_serial_number(serial as u32));

    manufacturer_info["Model year"]
        .as_u64()
        .map(|year| edid.set_week_year(EDIDWeekYear::ModelYear(year as u16)));

    let year_val = manufacturer_info["Year of manufacture"]
        .as_u64();

    match manufacturer_info["Week of manufacture"].as_u64() {
        Some(week) => {
            let year = year_val.expect("Couldn't decode the Year of Manufacture") as u16;
            edid.set_week_year(EDIDWeekYear::WeekYearOfManufacture(week as u8, year));
        },
        None => {
            year_val.map(|year| edid.set_week_year(EDIDWeekYear::YearOfManufacture(year as u16)));
        },
    }
}

fn decode_basic_display_parameters(edid: &mut EDID, basic_display: &Value) {
    let input_str = basic_display["Video input type"]
        .as_str()
        .expect("Couldn't decode video input type");
    let input = match input_str {
        "Digital" => {
            let color_type_str = basic_display["Display color type"]
                .as_str()
                .expect("Couldn't decode the color type");
            let color_type = match color_type_str {
                "RGB 4:4:4" => EDIDDisplayColorEncoding::RGB444,
                "RGB 4:4:4 + YCrCb 4:2:2" => EDIDDisplayColorEncoding::RGB444YCbCr422,
                "RGB 4:4:4 + YCrCb 4:4:4" => EDIDDisplayColorEncoding::RGB444YCbCr444,
                "RGB 4:4:4 + YCrCb 4:4:4 + YCrCb 4:2:2" => EDIDDisplayColorEncoding::RGB444YCbCr444YCbCr422,
                _ => panic!("Unknown Color Encoding"),
            };
            edid.set_display_color_type_encoding(EDIDDisplayColorTypeEncoding::ColorEncoding(color_type));
            
            let bpc_str = basic_display["Color Bit Depth"]
                .as_str();
            let bpc = match bpc_str {
                Some(bpc_str_inner) => {
                    match bpc_str_inner {
                        "6 Bits per Primary Color" => EDIDVideoDigitalColorDepth::Depth6bpc,
                        "8 Bits per Primary Color" => EDIDVideoDigitalColorDepth::Depth8bpc,
                        "10 Bits per Primary Color" => EDIDVideoDigitalColorDepth::Depth10bpc,
                        _ => panic!("Unknown bits per color"),
                    }
                },
                None => EDIDVideoDigitalColorDepth::Undefined,
            };

            let interface_str = basic_display["Digital Video Interface Standard Support"]
                .as_str();
            let interface = match interface_str {
                Some(interface_str_inner) => {
                    match interface_str_inner {
                        "DisplayPort" => EDIDVideoDigitalInterfaceStandard::DisplayPort,
                        "HDMI-a" => EDIDVideoDigitalInterfaceStandard::HDMIa,
                        _ => panic!("Unknown interface standard"),
                    }
                },
                None => EDIDVideoDigitalInterfaceStandard::Undefined,
            };

            EDIDVideoInput::Digital(EDIDVideoDigitalInterface::new(interface, bpc))
        }
        _ => panic!("Unknown interface type"),
    };
    edid.set_input(input);

    let cf_str = basic_display["Continuous frequency supported"]
        .as_bool()
        .expect("Couldn't decode continous frequency");
    edid.set_continuous_frequency(cf_str);

    let dpm_active_off_str = basic_display["DPM active-off supported"]
        .as_bool()
        .expect("Couldn't decode DPM active-off");
    edid.set_dpm_active_off(dpm_active_off_str);

    let dpm_standby_str = basic_display["DPM standby supported"]
        .as_bool()
        .expect("Couldn't decode DPM standby");
    edid.set_dpm_standby(dpm_standby_str);

    let dpm_suspend_str = basic_display["DPM suspend supported"]
        .as_bool()
        .expect("Couldn't decode DPM suspend");
    edid.set_dpm_suspend(dpm_suspend_str);

    let srgb_default_str = basic_display["sRGB Standard is default colour space"]
        .as_bool()
        .expect("Couldn't decode sRGB Standard Default");
    edid.set_srgb_default(srgb_default_str);
 
    let preferred_native_str = basic_display["Preferred timing includes native timing pixel format and refresh rate"]
        .as_bool()
        .expect("Couldn't decode Preferred timings");
    edid.set_preferred_timings_native(preferred_native_str);

    let gamma_str = basic_display["Display gamma"]
        .as_f64()
        .expect("Couldn't decode gamma") as f32;
    edid.set_gamma(gamma_str);

    basic_display["Aspect ratio (portrait)"]
        .as_str()
        .map(|val_str| {
            let mut split = val_str.split(":"); 

            let num = split.next()
                .expect("Couldn't decode the ratio numerator")
                .parse::<f32>()
                .expect("Couldn't parse the ratio numerator");

            let denum = split.next()
                .expect("Couldn't decode the ratio denominator")
                .parse::<f32>()
                .expect("Couldn't parse the ratio denominator");

            edid.set_screen_size_ratio(EDIDScreenSizeRatio::PortraitRatio(num / denum))
        });

    basic_display["Aspect ratio (landscape)"]
        .as_str()
        .map(|val_str| {
            let mut split = val_str.split(":");

            let num = split.next()
                .expect("Couldn't decode the ratio numerator")
                .trim()
                .parse::<f32>()
                .expect("Couldn't parse the ratio numerator");

            let denum = split.next()
                .expect("Couldn't decode the ratio denominator")
                .trim()
                .parse::<f32>()
                .expect("Couldn't parse the ratio denominator");

            edid.set_screen_size_ratio(EDIDScreenSizeRatio::LandscapeRatio(num / denum));
        });

    basic_display["Maximum dimensions (cm)"]
        .as_object()
        .map(|size| {
            let x_val = size["x"]
                .as_u64()
                .expect("Couldn't decode X screen size") as u8;
            let y_val = size["y"]
                .as_u64()
                .expect("Couldn't decode Y screen size") as u8;
            edid.set_screen_size_ratio(EDIDScreenSizeRatio::Size(x_val, y_val));
        });
}

fn decode_chromaticity(edid: &mut EDID, chromaticity: &Value) {
    chromaticity["Blue"]
        .as_object()
        .map(|chroma| {
            let x_val = chroma["x"]
            .as_u64()
            .expect("Couldn't decode X chroma value") as u16;
        let y_val = chroma["y"]
            .as_u64()
            .expect("Couldn't decode Y chroma value") as u16;

            edid.set_chroma_coordinates(EDIDChromaCoordinate::Blue, x_val, y_val);
        });

    chromaticity["Red"]
        .as_object()
        .map(|chroma| {
            let x_val = chroma["x"]
            .as_u64()
            .expect("Couldn't decode X chroma value") as u16;
        let y_val = chroma["y"]
            .as_u64()
            .expect("Couldn't decode Y chroma value") as u16;

            edid.set_chroma_coordinates(EDIDChromaCoordinate::Red, x_val, y_val);
        });

    chromaticity["Green"]
        .as_object()
        .map(|chroma| {
            let x_val = chroma["x"]
            .as_u64()
            .expect("Couldn't decode X chroma value") as u16;
        let y_val = chroma["y"]
            .as_u64()
            .expect("Couldn't decode Y chroma value") as u16;

            edid.set_chroma_coordinates(EDIDChromaCoordinate::Green, x_val, y_val);
        });

    chromaticity["White"]
        .as_object()
        .map(|chroma| {
            let x_val = chroma["x"]
            .as_u64()
            .expect("Couldn't decode X chroma value") as u16;
        let y_val = chroma["y"]
            .as_u64()
            .expect("Couldn't decode Y chroma value") as u16;

            edid.set_chroma_coordinates(EDIDChromaCoordinate::White, x_val, y_val);
        });
}

fn decode_established_timings(edid: &mut EDID, timings: &Value) {
    let map = timings.as_object()
        .expect("Couldn't decode the established timings section");

    for (timing, timing_val) in map.iter() {
        let et = match timing.as_str() {
            "1024x768 @ 60 Hz" => EDIDEstablishedTiming::ET_1024_768_60Hz,
            "1024x768 @ 72 Hz" => EDIDEstablishedTiming::ET_1024_768_70Hz,
            "1024x768 @ 75 Hz" => EDIDEstablishedTiming::ET_1024_768_75Hz,
            "1024x768 @ 87 Hz, interlaced (1024x768i)" => EDIDEstablishedTiming::ET_1024_768_87Hz_Interlaced,
            "1152x870 @ 75 Hz (Apple Macintosh II)" => EDIDEstablishedTiming::ET_1152_870_75Hz,
            "1280x1024 @ 75 Hz" => EDIDEstablishedTiming::ET_1280_1024_75Hz,
            "640x480 @ 60 Hz" => EDIDEstablishedTiming::ET_640_480_60Hz,
            "640x480 @ 67 Hz" => EDIDEstablishedTiming::ET_640_480_67Hz,
            "640x480 @ 72 Hz" => EDIDEstablishedTiming::ET_640_480_72Hz,
            "640x480 @ 75 Hz" => EDIDEstablishedTiming::ET_640_480_75Hz,
            "720x400 @ 70 Hz" => EDIDEstablishedTiming::ET_720_400_70Hz,
            "720x400 @ 88 Hz" => EDIDEstablishedTiming::ET_720_400_88Hz,
            "800x600 @ 56 Hz" => EDIDEstablishedTiming::ET_800_600_56Hz,
            "800x600 @ 60 Hz" => EDIDEstablishedTiming::ET_800_600_60Hz,
            "800x600 @ 72 Hz" => EDIDEstablishedTiming::ET_800_600_72Hz,
            "800x600 @ 75 Hz" => EDIDEstablishedTiming::ET_800_600_75Hz,
            "832x624 @ 75 Hz" => EDIDEstablishedTiming::ET_832_624_75Hz,

            // FIXME: Support manufacturer specific display code
            "Manufacturer specific display mode 1" => continue,
            "Manufacturer specific display mode 2" => continue,
            "Manufacturer specific display mode 3" => continue,
            "Manufacturer specific display mode 4" => continue,
            "Manufacturer specific display mode 5" => continue,
            "Manufacturer specific display mode 6" => continue,
            "Manufacturer specific display mode 7" => continue,

            _ => panic!("Couldn't decode the established timing key"),
        };

        timing_val.as_bool()
            .map(|val| if val {
                edid.add_established_timing(et);
            });
    }
}

fn decode_standard_timings(edid: &mut EDID, timings: &Value) {
    let list = timings.as_array()
        .expect("Couldn't decode Standard timings list");

    for timing in list {
        let item = timing.as_object()
            .expect("Couldn't decode Standard Timing Item");

        let frequency = item["Frequency"]
            .as_u64()
            .expect("Couldn't decode Standard Timing Frequency") as u8;

        let ratio_str = item["Ratio"]
            .as_str()
            .expect("Couldn't decode Standard Timing ratio");

        let ratio = match ratio_str {
            "16:10" => EDIDStandardTimingRatio::Ratio_16_10,
            "4:3" => EDIDStandardTimingRatio::Ratio_4_3,
            "5:4" => EDIDStandardTimingRatio::Ratio_5_4,
            "16:9" => EDIDStandardTimingRatio::Ratio_16_9,
            _ => panic!("Couldn't decode Standard Timing ratio"),
        };

        let x = item["X resolution"]
            .as_u64()
            .expect("Couldn't decode Standard Timing Resolution") as u16;

        edid.add_standard_timing(EDIDStandardTiming::new(x, ratio, frequency));
    }
}

fn decode_descriptor_dtd(edid: &mut EDID, desc: &Value) {
    let addressable = desc["Addressable"]
        .as_object()
        .expect("Couldn't decode Descriptor Addressable section");

    let hdisplay = addressable["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Addressable X size") as u16;

    let vdisplay = addressable["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Addressable Y size") as u16;

    let blanking = desc["Blanking"]
        .as_object()
        .expect("Couldn't decode Descriptor Blanking section");

    let hblank = blanking["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Blanking X size") as u16;

    let vblank = blanking["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Blanking y size") as u16;
    
    let fp = desc["Front porch"]
        .as_object()
        .expect("Couldn't decode Descriptor Front Porch section");

    let hfp = fp["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Front Porch X size") as u16;

    let vfp = fp["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Front Porch y size") as u16;

    let sync = desc["Sync pulse"]
        .as_object()
        .expect("Couldn't decode Descriptor Sync Pulse section");

    let hsync = sync["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Sync X size") as u16;

    let vsync = sync["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Sync y size") as u16;

    let size = desc["Image size (mm)"]
        .as_object()
        .expect("Couldn't decode Descriptor Sync Pulse section");

    let hsize = size["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Size X size") as u16;

    let vsize = size["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Size y size") as u16;

    let interlace = desc["Interlace"]
        .as_bool()
        .expect("Couldn't decode Descriptor interlace");

    let pixel_clock_mhz = desc["Pixel clock (MHz)"]
        .as_f64()
        .expect("Couldn't decode Descriptor Pixel Clock");
        
    let pixel_clock = (pixel_clock_mhz * 1000.0) as u32;

    let stereo_str = desc["Stereo viewing"]
        .as_str()
        .expect("Couldn't decode Descriptor stereo");
    let stereo = match stereo_str {
        "No stereo" => EDIDDetailedTimingStereo::None,
        "Field sequential stereo, right image when stereo sync signal = 1" => EDIDDetailedTimingStereo::FieldSequentialRightOnSync,
        _ => panic!("Couldn't decode Descriptor stereo"),
    };

    let sync_type = desc["Sync type"]
        .as_object()
        .expect("Couldn't decode Descriptor Sync Type section");

    let sync_type_type_str = sync_type["Type"]
        .as_str()
        .expect("Couldn't decode the Descriptor Sync Type");
    let sync_type_type = match sync_type_type_str {
        "Analog Composite Sync" => {
            let serrations = sync_type["Serrations"]
                .as_bool()
                .expect("Couldn't decode Serrations");

            let sync_on_rgb = sync_type["Sync on RGB"]
                .as_bool()
                .expect("Couldn't decode Sync on RGB");

            let analog_sync = EDIDDetailedTimingAnalogSync::Composite(serrations, sync_on_rgb);
            EDIDDetailedTimingSync::Analog(analog_sync)
        },
        "Digital Separate Sync" => {
            let vpol_str = sync_type["Vertical sync"]
                .as_str()
                .expect("Couldn't decode Descriptor Vertical Sync polarity");
            let vpol = match vpol_str {
                "Negative" => false,
                "Positive" => true,
                _ => panic!("Couldn't decode Descriptor Vertical Sync polarity"),
            };

            let hpol_str = sync_type["Horizontal sync (outside of V-sync)"]
                .as_str()
                .expect("Couldn't decode Descriptor Horizontal Sync polarity");
            let hpol = match hpol_str {
                    "Negative" => false,
                    "Positive" => true,
                    _ => panic!("Couldn't decode Descriptor Vertical Sync polarity"),
                };

            let digital_sync = EDIDDetailedTimingDigitalSync::Separate(hpol, vpol);
            EDIDDetailedTimingSync::Digital(digital_sync)
        },
        _ => panic!("Couldn't Decode Descriptor Sync Type"),
    };

    edid.add_descriptor(EDIDDescriptor::DetailedTiming(
        EDIDDetailedTiming::new()
            .set_display(hdisplay, vdisplay)
            .set_interlace(interlace)
            .set_front_porch(hfp, vfp)
            .set_blanking(hblank, vblank)
            .set_sync_pulse(hsync, vsync)
            .set_sync_type(sync_type_type)
            .set_pixel_clock(pixel_clock)
            .set_stereo(stereo)
            .set_size(hsize, vsize)));
}

fn decode_display_range(edid: &mut EDID, desc: &Value) {
    let hrate = desc["Horizontal rate (kHz)"]
        .as_object()
        .expect("Couldn't decode Display Range Horizontal section");

    let hrate_min = hrate["Minimum"]
        .as_u64()
        .expect("Couldn't decode Display Range Minimum Horizontal frequency") as u16;

    let hrate_max = hrate["Maximum"]
        .as_u64()
        .expect("Couldn't decode Display Range Maximum Horizontal frequency") as u16;

    let vrate = desc["Vertical rate (Hz)"]
        .as_object()
        .expect("Couldn't decode Display Range Vertical section");

    let vrate_min = vrate["Minimum"]
        .as_u64()
        .expect("Couldn't decode Display Range Minimum Vertical frequency") as u16;

    let vrate_max = vrate["Maximum"]
        .as_u64()
        .expect("Couldn't decode Display Range Maximum Vertical frequency") as u16;

    let pixel_clock = desc["Pixel clock (MHz)"]
        .as_f64()
        .expect("Couldn't decode the Display Range Maximum Pixel Frequency") as u16;

    let subtype_str = desc["Subtype"]
        .as_str()
        .expect("Couldn't decode the Display Range Subtype");

    let subtype = match subtype_str {
        "Default GTF supported" => EDIDDisplayRangeLimitsSubtype::DefaultGTF,
        "Range Limits Only - no additional info" => EDIDDisplayRangeLimitsSubtype::RangeLimitsOnly,
        _ => panic!("Couldn't decode the Display Range Subtype"),
    };

    edid.add_descriptor(EDIDDescriptor::DisplayRangeLimits(
        EDIDDisplayRangeLimits::new()
            .set_horizontal_rate_range(hrate_min, hrate_max)
            .set_vertical_rate_range(vrate_min, vrate_max)
            .set_pixel_clock_max(pixel_clock)
            .set_subtype(subtype)
    ));
}

fn decode_data_string(edid: &mut EDID, desc: &Value) {
    let string = desc["Data string"].as_str()
        .expect("Couldn't decode Product Name")
        .to_string();

    edid.add_descriptor(EDIDDescriptor::DataString(string));
}

fn decode_descriptor_name(edid: &mut EDID, desc: &Value) {
    let name = desc["Data string"].as_str()
        .expect("Couldn't decode Product Name")
        .to_string();

    edid.add_descriptor(EDIDDescriptor::ProductName(name));
}

fn decode_descriptor_serial(edid: &mut EDID, desc: &Value) {
    let serial = desc["Data string"].as_str()
        .expect("Couldn't decode Product Name")
        .to_string();

    edid.add_descriptor(EDIDDescriptor::ProductSerialNumber(serial));

}

fn decode_custom_descriptor(edid: &mut EDID, desc: &Value) {
    let tag = desc["Tag"]
        .as_u64()
        .expect("Couldn't decode the custom descriptor's tag") as u8;

    let data = desc["Blob"]
        .as_array()
        .expect("Couldn't decode Product Name")
        .into_iter()
        .map(|val| val.as_u64().expect("Couldn't decode blob") as u8)
        .collect();
    
    edid.add_descriptor(EDIDDescriptor::Custom(tag, data));
}

fn decode_descriptors(edid: &mut EDID, descriptors: &Value) {
    let list = descriptors.as_array()
        .expect("Couldn't decode Descriptors list");

    for desc in list {
        let desc_type = desc["Type"].as_str()
            .expect("Couldn't decode descriptor's type");

        match desc_type {
            "Alphanumeric Data String (ASCII)" => decode_data_string(edid, desc),
            "Detailed Timing Descriptor" => decode_descriptor_dtd(edid, desc),
            "Display Range Limits Descriptor" => decode_display_range(edid, desc),
            "Display Product Name" => decode_descriptor_name(edid, desc),
            "Display Product Serial Number" => decode_descriptor_serial(edid, desc),
            "Dummy descriptor" => edid.add_descriptor(EDIDDescriptor::Dummy),
            "Manufacturer Specified Display Descriptor" => decode_custom_descriptor(edid, desc),
            _ => panic!("Couldn't decode the descriptor's type"),
        };
    }
}

fn decode_base_edid(edid: &mut EDID, json: &Value) {
    let manufacturer_info = &json["Manufacturer Info"];
    decode_manufacturer_info(edid, manufacturer_info);

    let basic_display = &json["Basic Display"];
    decode_basic_display_parameters(edid, basic_display);

    let chromaticity = &json["Chromaticity"];
    decode_chromaticity(edid, chromaticity);

    let established_timings = &json["Established Timing"];
    decode_established_timings(edid, established_timings);

    let standard_timings = &json["Standard Timing"];
    decode_standard_timings(edid, standard_timings);

    let descriptors = &json["Descriptors"];
    decode_descriptors(edid, descriptors);
}

fn edid_equals(val1: &[u8], val2: &[u8]) -> bool {
    if val1.len() != val2.len() {
        return false;
    }

    for i in 0..val1.len() {
        if val1[i] == val2[i] {
            continue;
        }

        if (i > 0x36) && ((i - 0x36) % 18) == 17 {
            if ((val1[i] ^ val2[i]) & 0x61) == 1 {
                continue;
            }
        }

        if i >= 0x26 && i < 0x36 {
            let mut st1 = [0; 16];
            st1.copy_from_slice(&val1[0x26..0x36]);
            st1.sort();

            let mut st2 = [0; 16];
            st2.copy_from_slice(&val2[0x26..0x36]);
            st2.sort();

            if st1 == st2 {
                continue;
            }
        }

        println!("Index {:#x} is different: {:#x} vs {:#x}", i, val1[i], val2[i]);
        return false;
    }

    true
}

#[test_resources("tests/edid-db/*.bin")]
fn test_edid(edid: &str) {
    let output = Command::new("tests/tools/edid-chamelium/edid2json.py")
        .arg(edid)
        .output()
        .expect("Couldn't decode the EDID");

    assert!(output.status.success());

    let mut input_file = File::open(edid).unwrap();
    let mut input_data: [u8; 0x80] = [0; 0x80];
    input_file.read_exact(&mut input_data).unwrap();

    let mut output_data: Vec<u8> = Vec::with_capacity(0x80);

    let output_str = std::str::from_utf8(&output.stdout)
        .expect("Couldn't convert the output to UTF-8");

    let json: Value = serde_json::from_str(output_str).expect("Couldn't parse the JSON output");

    assert!(json["Version"] == "1.4");

    let mut edid = EDID::new(EDIDVersion::V1R4);

    let base = &json["Base"];
    decode_base_edid(&mut edid, base);

    edid.serialize(&mut output_data);

    assert!(edid_equals(&input_data[0..0x7e], &output_data.as_slice()[0..0x7e]));
}
