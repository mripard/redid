use std::{convert::TryInto, fs::File, io::Read, process::Command, str::FromStr};

use num_traits::ToPrimitive;
use serde_json::Value;
use test_generator::test_resources;

use redid::{
    EdidAnalogSignalLevelStandard, EdidAnalogVideoInputDefinition, EdidAnalogVideoSetup,
    EdidChromaticityPoint, EdidChromaticityPoints, EdidDescriptorCustom,
    EdidDescriptorDetailedTiming, EdidDescriptorString, EdidDetailedTimingAnalogSync,
    EdidDetailedTimingDigitalCompositeSync, EdidDetailedTimingDigitalSeparateSync,
    EdidDetailedTimingDigitalSync, EdidDetailedTimingDigitalSyncKind, EdidDetailedTimingStereo,
    EdidDetailedTimingSync, EdidDisplayColorType, EdidDisplayRangeHorizontalFreq,
    EdidDisplayRangePixelClock, EdidDisplayRangeVerticalFreq, EdidDisplayRangeVideoTimingsGTF,
    EdidDisplayRangeVideoTimingsGTFStartFrequency, EdidDisplayTransferCharacteristics,
    EdidEstablishedTiming, EdidFilterChromaticity, EdidManufactureDate, EdidManufacturer,
    EdidProductCode, EdidR3BasicDisplayParametersFeatures, EdidR3Descriptor,
    EdidR3DigitalVideoInputDefinition, EdidR3DisplayRangeLimits,
    EdidR3DisplayRangeVideoTimingsSupport, EdidR3FeatureSupport, EdidR3ImageSize,
    EdidR3VideoInputDefinition, EdidR4BasicDisplayParametersFeatures, EdidR4Date, EdidR4Descriptor,
    EdidR4DescriptorEstablishedTimings, EdidR4DescriptorEstablishedTimingsIII,
    EdidR4DigitalColorDepth, EdidR4DigitalInterface, EdidR4DigitalVideoInputDefinition,
    EdidR4DisplayColor, EdidR4DisplayColorEncoding, EdidR4DisplayRangeHorizontalFreq,
    EdidR4DisplayRangeLimits, EdidR4DisplayRangeVerticalFreq,
    EdidR4DisplayRangeVideoTimingsAspectRatio, EdidR4DisplayRangeVideoTimingsCVT,
    EdidR4DisplayRangeVideoTimingsCVTR1, EdidR4DisplayRangeVideoTimingsSupport,
    EdidR4FeatureSupport, EdidR4ImageSize, EdidR4ManufactureDate, EdidR4VideoInputDefinition,
    EdidRelease3, EdidRelease4, EdidScreenSize, EdidSerialNumber, EdidStandardTiming,
    EdidStandardTimingHorizontalSize, EdidStandardTimingRatio, EdidStandardTimingRefreshRate,
    IntoBytes,
};
use uom::si::{f32::Frequency, frequency::kilohertz};

fn decode_manufacturer_name(manufacturer_info: &Value) -> EdidManufacturer {
    manufacturer_info["Manufacturer ID"]
        .as_str()
        .expect("Couldn't decode the manufacturer ID")
        .try_into()
        .unwrap()
}

fn decode_product_code(manufacturer_info: &Value) -> EdidProductCode {
    let code = manufacturer_info["ID Product Code"]
        .as_u64()
        .expect("Couldn't decode the product ID") as u16;

    code.into()
}

fn decode_serial_number(manufacturer_info: &Value) -> Option<EdidSerialNumber> {
    manufacturer_info["Serial number"]
        .as_u64()
        .map(|v| (v as u32).into())
}

fn decode_date_release_3(manufacturer_info: &Value) -> EdidManufactureDate {
    let year = manufacturer_info["Year of manufacture"].as_u64().unwrap() as u16;

    if let Some(val) = manufacturer_info["Week of manufacture"].as_u64() {
        (val as u8, year).try_into().unwrap()
    } else {
        year.try_into().unwrap()
    }
}

fn decode_date_release_4(manufacturer_info: &Value) -> EdidR4Date {
    if let Some(year) = manufacturer_info["Model year"].as_u64() {
        EdidR4Date::Model((year as u16).try_into().unwrap())
    } else {
        let year = manufacturer_info["Year of manufacture"].as_u64().unwrap() as u16;

        let date: EdidR4ManufactureDate =
            if let Some(val) = manufacturer_info["Week of manufacture"].as_u64() {
                (val as u8, year).try_into().unwrap()
            } else {
                year.try_into().unwrap()
            };

        EdidR4Date::Manufacture(date)
    }
}

fn decode_analog_input(basic_display: &Value) -> EdidAnalogVideoInputDefinition {
    let signal_level_str = basic_display["Video white and sync levels"]
        .as_str()
        .expect("Couldn't decode the sync level");
    let signal_level = match signal_level_str {
        "+0.7/0 V" => EdidAnalogSignalLevelStandard::V_0_700_S_0_000_T_0_700,
        "+0.7/-0.3 V" => EdidAnalogSignalLevelStandard::V_0_700_S_0_300_T_1_000,
        "+0.714/-0.286 V" => EdidAnalogSignalLevelStandard::V_0_714_S_0_286_T_1_000,
        "+1.0/-0.4 V" => EdidAnalogSignalLevelStandard::V_1_000_S_0_400_T_1_400,
        _ => panic!("Unknown Sync Level"),
    };

    let blank_to_black_setup = basic_display["Blank-to-black setup expected"]
        .as_bool()
        .expect("Couldn't decode blank-to-black");
    let setup = if blank_to_black_setup {
        EdidAnalogVideoSetup::BlankToBlackSetupOrPedestal
    } else {
        EdidAnalogVideoSetup::BlankLevelIsBlackLevel
    };

    let separate_sync = basic_display["Separate sync supported"]
        .as_bool()
        .expect("Couldn't decode separate sync");

    let sync_on_h = basic_display["Composite sync (on HSync) supported"]
        .as_bool()
        .expect("Couldn't decode sync on Hsync");

    let sync_on_green = basic_display["Sync on green supported"]
        .as_bool()
        .expect("Couldn't decode Sync on green");

    let vsync_serrations = basic_display["VSync serrated when composite/sync-on-green used"]
        .as_bool()
        .expect("Couldn't decode Vsync serrations");

    EdidAnalogVideoInputDefinition::builder()
        .signal_level(signal_level)
        .setup(setup)
        .separate_hv_sync_signals(separate_sync)
        .composite_sync_signal_on_hsync(sync_on_h)
        .composite_sync_signal_on_green_video(sync_on_green)
        .serrations_on_vsync(vsync_serrations)
        .build()
}

fn decode_digital_input_release_3(basic_display: &Value) -> EdidR3DigitalVideoInputDefinition {
    let dfp_bool = basic_display["Digital Video Interface Standard Support"]
        .as_str()
        .map(|s| if s == "DVI" { true } else { false })
        .unwrap_or_default();

    EdidR3DigitalVideoInputDefinition::builder()
        .dfp1_compatible(dfp_bool)
        .build()
}

fn decode_video_input_release_3(basic_display: &Value) -> EdidR3VideoInputDefinition {
    let input_str = basic_display["Video input type"]
        .as_str()
        .expect("Couldn't decode video input type");

    match input_str {
        "Analog" => EdidR3VideoInputDefinition::Analog(decode_analog_input(basic_display)),
        "Digital" => {
            EdidR3VideoInputDefinition::Digital(decode_digital_input_release_3(basic_display))
        }
        _ => panic!("Unknown interface type"),
    }
}

fn decode_size_release_3(basic_display: &Value) -> EdidR3ImageSize {
    if let Some(val) = basic_display["Maximum dimensions (cm)"].as_object() {
        let x_val = val["x"]
            .as_u64()
            .expect("Couldn't decode X screen size")
            .to_u8()
            .unwrap()
            .try_into()
            .unwrap();
        let y_val = val["y"]
            .as_u64()
            .expect("Couldn't decode Y screen size")
            .to_u8()
            .unwrap()
            .try_into()
            .unwrap();

        return EdidR3ImageSize::Size(
            EdidScreenSize::builder()
                .horizontal_cm(x_val)
                .vertical_cm(y_val)
                .build(),
        );
    };

    EdidR3ImageSize::Undefined
}

fn decode_display_type_release_3(basic_display: &Value) -> EdidDisplayColorType {
    let input_str = basic_display["Video input type"]
        .as_str()
        .expect("Couldn't decode video input type");

    match input_str {
        "Analog" => {
            let display_type_str = basic_display["Display color type"]
                .as_str()
                .expect("Couldn't decode the display color type");

            match display_type_str {
                "Monochrome/Grayscale" => EdidDisplayColorType::MonochromeGrayScale,
                "RGB color" => EdidDisplayColorType::RGBColor,
                _ => panic!("Unknown Display Type"),
            }
        }
        "Digital" => {
            let color_type_str = basic_display["Display color type"]
                .as_str()
                .expect("Couldn't decode the color type");

            match color_type_str {
                "RGB 4:4:4" => EdidDisplayColorType::MonochromeGrayScale,
                "RGB 4:4:4 + YCrCb 4:4:4" => EdidDisplayColorType::RGBColor,
                "RGB 4:4:4 + YCrCb 4:4:4 + YCrCb 4:2:2" => EdidDisplayColorType::Undefined,
                _ => panic!("Unknown Display Type"),
            }
        }
        _ => panic!("Unknown interface type"),
    }
}

fn decode_feature_support_release_3(basic_display: &Value) -> EdidR3FeatureSupport {
    let gtf_bool = basic_display["Continuous frequency supported"]
        .as_bool()
        .expect("Couldn't decode continous frequency");

    let dpm_active_off_bool = basic_display["DPM active-off supported"]
        .as_bool()
        .expect("Couldn't decode DPM active-off");

    let dpm_standby_bool = basic_display["DPM standby supported"]
        .as_bool()
        .expect("Couldn't decode DPM standby");

    let dpm_suspend_bool = basic_display["DPM suspend supported"]
        .as_bool()
        .expect("Couldn't decode DPM suspend");

    let srgb_default_bool = basic_display["sRGB Standard is default colour space"]
        .as_bool()
        .expect("Couldn't decode sRGB Standard Default");

    EdidR3FeatureSupport::builder()
        .display_type(decode_display_type_release_3(basic_display))
        .default_gtf_supported(gtf_bool)
        .active_off_is_very_low_power(dpm_active_off_bool)
        .standby(dpm_standby_bool)
        .suspend(dpm_suspend_bool)
        .srgb_default_color_space(srgb_default_bool)
        .build()
}

fn decode_basic_display_release_3(basic_display: &Value) -> EdidR3BasicDisplayParametersFeatures {
    let gamma_val = basic_display["Display gamma"]
        .as_f64()
        .expect("Couldn't decode gamma") as f32;

    let gamma = if gamma_val == 3.55 {
        EdidDisplayTransferCharacteristics::DisplayInformationExtension(())
    } else {
        gamma_val.try_into().unwrap()
    };

    EdidR3BasicDisplayParametersFeatures::builder()
        .video_input(decode_video_input_release_3(basic_display))
        .size(decode_size_release_3(basic_display))
        .display_transfer_characteristic(gamma)
        .feature_support(decode_feature_support_release_3(basic_display))
        .build()
}

fn decode_digital_input_release_4(basic_display: &Value) -> EdidR4DigitalVideoInputDefinition {
    let input = EdidR4DigitalVideoInputDefinition::builder();

    let bpc_str = basic_display["Color Bit Depth"].as_str();
    let bpc = match bpc_str {
        Some(bpc_str_inner) => match bpc_str_inner {
            "6 Bits per Primary Color" => EdidR4DigitalColorDepth::Depth6Bpc,
            "8 Bits per Primary Color" => EdidR4DigitalColorDepth::Depth8Bpc,
            "10 Bits per Primary Color" => EdidR4DigitalColorDepth::Depth10Bpc,
            "12 Bits per Primary Color" => EdidR4DigitalColorDepth::Depth12Bpc,
            _ => panic!("Unknown bits per color"),
        },
        None => EdidR4DigitalColorDepth::DepthUndefined,
    };
    let input = input.color_depth(bpc);

    let interface_str = basic_display["Digital Video Interface Standard Support"].as_str();
    let interface = match interface_str {
        Some(interface_str_inner) => match interface_str_inner {
            "DisplayPort" => EdidR4DigitalInterface::DisplayPort,
            "DVI" => EdidR4DigitalInterface::DVI,
            "HDMI-a" => EdidR4DigitalInterface::HDMIa,
            "HDMI-b" => EdidR4DigitalInterface::HDMIb,
            _ => panic!("Unknown interface standard"),
        },
        None => EdidR4DigitalInterface::Undefined,
    };
    let input = input.interface(interface);

    input.build()
}

fn decode_video_input_release_4(basic_display: &Value) -> EdidR4VideoInputDefinition {
    let input_str = basic_display["Video input type"]
        .as_str()
        .expect("Couldn't decode video input type");

    match input_str {
        "Analog" => EdidR4VideoInputDefinition::Analog(decode_analog_input(basic_display)),
        "Digital" => {
            EdidR4VideoInputDefinition::Digital(decode_digital_input_release_4(basic_display))
        }
        _ => panic!("Unknown interface type"),
    }
}

fn decode_size_release_4(basic_display: &Value) -> EdidR4ImageSize {
    if let Some(val) = basic_display["Aspect ratio (portrait)"].as_str() {
        let mut split = val.split(":");

        let num = split
            .next()
            .expect("Couldn't decode the ratio numerator")
            .trim()
            .parse::<f32>()
            .expect("Couldn't parse the ratio numerator");

        let denum = split
            .next()
            .expect("Couldn't decode the ratio denominator")
            .trim()
            .parse::<f32>()
            .expect("Couldn't parse the ratio denominator");

        let ratio = (num, denum).try_into().unwrap();
        return EdidR4ImageSize::PortraitRatio(ratio);
    };

    if let Some(val) = basic_display["Aspect ratio (landscape)"].as_str() {
        let mut split = val.split(":");

        let num = split
            .next()
            .expect("Couldn't decode the ratio numerator")
            .trim()
            .parse::<f32>()
            .expect("Couldn't parse the ratio numerator");

        let denum = split
            .next()
            .expect("Couldn't decode the ratio denominator")
            .trim()
            .parse::<f32>()
            .expect("Couldn't parse the ratio denominator");

        let ratio = (num, denum).try_into().unwrap();
        return EdidR4ImageSize::LandscapeRatio(ratio);
    };

    if let Some(val) = basic_display["Maximum dimensions (cm)"].as_object() {
        let x_val = val["x"]
            .as_u64()
            .expect("Couldn't decode X screen size")
            .to_u8()
            .unwrap()
            .try_into()
            .unwrap();
        let y_val = val["y"]
            .as_u64()
            .expect("Couldn't decode Y screen size")
            .to_u8()
            .unwrap()
            .try_into()
            .unwrap();

        return EdidR4ImageSize::Size(
            EdidScreenSize::builder()
                .horizontal_cm(x_val)
                .vertical_cm(y_val)
                .build(),
        );
    };

    EdidR4ImageSize::Undefined
}

fn decode_display_color_release_4(basic_display: &Value) -> EdidR4DisplayColor {
    let input_str = basic_display["Video input type"]
        .as_str()
        .expect("Couldn't decode video input type");

    match input_str {
        "Analog" => {
            let display_type_str = basic_display["Display color type"]
                .as_str()
                .expect("Couldn't decode the display color type");

            EdidR4DisplayColor::Analog(match display_type_str {
                "Monochrome/Grayscale" => EdidDisplayColorType::MonochromeGrayScale,
                "RGB color" => EdidDisplayColorType::RGBColor,
                _ => panic!("Unknown Color Encoding"),
            })
        }
        "Digital" => {
            let color_type_str = basic_display["Display color type"]
                .as_str()
                .expect("Couldn't decode the color type");

            EdidR4DisplayColor::Digital(match color_type_str {
                "RGB 4:4:4" => EdidR4DisplayColorEncoding::RGB444,
                "RGB 4:4:4 + YCrCb 4:2:2" => EdidR4DisplayColorEncoding::RGB444YCbCr422,
                "RGB 4:4:4 + YCrCb 4:4:4" => EdidR4DisplayColorEncoding::RGB444YCbCr444,
                "RGB 4:4:4 + YCrCb 4:4:4 + YCrCb 4:2:2" => {
                    EdidR4DisplayColorEncoding::RGB444YCbCr444YCbCr422
                }
                _ => panic!("Unknown Color Encoding"),
            })
        }
        _ => panic!("Unknown interface type"),
    }
}

fn decode_feature_support_release_4(basic_display: &Value) -> EdidR4FeatureSupport {
    let cf_bool = basic_display["Continuous frequency supported"]
        .as_bool()
        .expect("Couldn't decode continous frequency");

    let dpm_active_off_bool = basic_display["DPM active-off supported"]
        .as_bool()
        .expect("Couldn't decode DPM active-off");

    let dpm_standby_bool = basic_display["DPM standby supported"]
        .as_bool()
        .expect("Couldn't decode DPM standby");

    let dpm_suspend_bool = basic_display["DPM suspend supported"]
        .as_bool()
        .expect("Couldn't decode DPM suspend");

    let srgb_default_bool = basic_display["sRGB Standard is default colour space"]
        .as_bool()
        .expect("Couldn't decode sRGB Standard Default");

    let preferred_native_bool = basic_display
        ["Preferred timing includes native timing pixel format and refresh rate"]
        .as_bool()
        .expect("Couldn't decode Preferred timings");

    #[allow(deprecated)]
    EdidR4FeatureSupport::builder()
        .color(decode_display_color_release_4(basic_display))
        .continuous_frequency(cf_bool)
        .active_off_is_very_low_power(dpm_active_off_bool)
        .standby(dpm_standby_bool)
        .suspend(dpm_suspend_bool)
        .srgb_default_color_space(srgb_default_bool)
        .preferred_timing_mode_is_native(preferred_native_bool)
        .build()
}

fn decode_basic_display_release_4(basic_display: &Value) -> EdidR4BasicDisplayParametersFeatures {
    let gamma_val = basic_display["Display gamma"]
        .as_f64()
        .expect("Couldn't decode gamma") as f32;

    let gamma = if gamma_val == 3.55 {
        EdidDisplayTransferCharacteristics::DisplayInformationExtension(())
    } else {
        gamma_val.try_into().unwrap()
    };

    EdidR4BasicDisplayParametersFeatures::builder()
        .video_input(decode_video_input_release_4(basic_display))
        .size(decode_size_release_4(basic_display))
        .display_transfer_characteristic(gamma)
        .feature_support(decode_feature_support_release_4(basic_display))
        .build()
}

fn decode_color_chromaticity(chroma: &serde_json::Map<String, Value>) -> EdidChromaticityPoint {
    let x_val = chroma["x"]
        .as_f64()
        .expect("Couldn't decode X chroma value") as f32
        / 1024.0;
    let y_val = chroma["y"]
        .as_f64()
        .expect("Couldn't decode Y chroma value") as f32
        / 1024.0;

    (x_val, y_val).try_into().unwrap()
}

fn decode_basic_display_chromaticity(
    basic_display: &Value,
    chromaticity: &Value,
) -> EdidFilterChromaticity {
    let mut monochrome = false;

    let input_str = basic_display["Video input type"]
        .as_str()
        .expect("Couldn't decode video input type");

    if input_str == "Analog" {
        let display_type_str = basic_display["Display color type"]
            .as_str()
            .expect("Couldn't decode the display color type");

        if display_type_str == "Monochrome/Grayscale" {
            monochrome = true;
        }
    }

    if !monochrome {
        let blue = chromaticity["Blue"].as_object().unwrap();
        let blue = decode_color_chromaticity(blue);

        let red = chromaticity["Red"].as_object().unwrap();
        let red = decode_color_chromaticity(red);

        let green = chromaticity["Green"].as_object().unwrap();
        let green = decode_color_chromaticity(green);

        let white = chromaticity["White"].as_object().unwrap();
        let white = decode_color_chromaticity(white);

        EdidFilterChromaticity::Color(
            EdidChromaticityPoints::builder()
                .blue(blue)
                .red(red)
                .green(green)
                .white(white)
                .build(),
        )
    } else {
        let white = chromaticity["White"].as_object().unwrap();
        let white = decode_color_chromaticity(white);

        EdidFilterChromaticity::MonoChrome(white)
    }
}

fn decode_established_timings(timings: &Value) -> Vec<EdidEstablishedTiming> {
    let mut list = Vec::new();

    let map = timings
        .as_object()
        .expect("Couldn't decode the established timings section");

    for (timing, timing_val) in map.iter() {
        let et = match timing.as_str() {
            "1024x768 @ 60 Hz" => EdidEstablishedTiming::ET_1024_768_60hz,
            "1024x768 @ 72 Hz" => EdidEstablishedTiming::ET_1024_768_70hz,
            "1024x768 @ 75 Hz" => EdidEstablishedTiming::ET_1024_768_75hz,
            "1024x768 @ 87 Hz, interlaced (1024x768i)" => {
                EdidEstablishedTiming::ET_1024_768_87hz_Interlaced
            }
            "1152x870 @ 75 Hz (Apple Macintosh II)" => EdidEstablishedTiming::ET_1152_870_75hz,
            "1280x1024 @ 75 Hz" => EdidEstablishedTiming::ET_1280_1024_75hz,
            "640x480 @ 60 Hz" => EdidEstablishedTiming::ET_640_480_60hz,
            "640x480 @ 67 Hz" => EdidEstablishedTiming::ET_640_480_67hz,
            "640x480 @ 72 Hz" => EdidEstablishedTiming::ET_640_480_72hz,
            "640x480 @ 75 Hz" => EdidEstablishedTiming::ET_640_480_75hz,
            "720x400 @ 70 Hz" => EdidEstablishedTiming::ET_720_400_70hz,
            "720x400 @ 88 Hz" => EdidEstablishedTiming::ET_720_400_88hz,
            "800x600 @ 56 Hz" => EdidEstablishedTiming::ET_800_600_56hz,
            "800x600 @ 60 Hz" => EdidEstablishedTiming::ET_800_600_60hz,
            "800x600 @ 72 Hz" => EdidEstablishedTiming::ET_800_600_72hz,
            "800x600 @ 75 Hz" => EdidEstablishedTiming::ET_800_600_75hz,
            "832x624 @ 75 Hz" => EdidEstablishedTiming::ET_832_624_75hz,
            "Manufacturer specific display mode 1" => EdidEstablishedTiming::Manufacturer6,
            "Manufacturer specific display mode 2" => EdidEstablishedTiming::Manufacturer5,
            "Manufacturer specific display mode 3" => EdidEstablishedTiming::Manufacturer4,
            "Manufacturer specific display mode 4" => EdidEstablishedTiming::Manufacturer3,
            "Manufacturer specific display mode 5" => EdidEstablishedTiming::Manufacturer2,
            "Manufacturer specific display mode 6" => EdidEstablishedTiming::Manufacturer1,
            "Manufacturer specific display mode 7" => EdidEstablishedTiming::Manufacturer0,

            _ => panic!("Couldn't decode the established timing key"),
        };

        if let Some(val) = timing_val.as_bool() {
            if val {
                list.push(et);
            }
        }
    }

    list
}

fn decode_standard_timings(timings: &Value) -> Vec<EdidStandardTiming> {
    let mut st = Vec::new();

    let list = timings
        .as_array()
        .expect("Couldn't decode Standard timings list");

    for timing in list {
        let item = timing
            .as_object()
            .expect("Couldn't decode Standard Timing Item");

        let frequency: EdidStandardTimingRefreshRate = (item["Frequency"]
            .as_u64()
            .expect("Couldn't decode Standard Timing Frequency")
            as u8)
            .try_into()
            .unwrap();

        let ratio_str = item["Ratio"]
            .as_str()
            .expect("Couldn't decode Standard Timing ratio");

        let ratio = match ratio_str {
            "16:10" => EdidStandardTimingRatio::Ratio_16_10,
            "4:3" => EdidStandardTimingRatio::Ratio_4_3,
            "5:4" => EdidStandardTimingRatio::Ratio_5_4,
            "16:9" => EdidStandardTimingRatio::Ratio_16_9,
            _ => panic!("Couldn't decode Standard Timing ratio"),
        };

        let x: EdidStandardTimingHorizontalSize = (item["X resolution"]
            .as_u64()
            .expect("Couldn't decode Standard Timing Resolution")
            as u16)
            .try_into()
            .unwrap();

        st.push(
            EdidStandardTiming::builder()
                .x(x)
                .ratio(ratio)
                .frequency(frequency)
                .build(),
        );
    }

    st
}

fn decode_descriptor_dtd(desc: &Value) -> EdidDescriptorDetailedTiming {
    let addressable = desc["Addressable"]
        .as_object()
        .expect("Couldn't decode Descriptor Addressable section");

    let hdisplay_raw = addressable["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Addressable X size") as u16;

    let hdisplay = hdisplay_raw.try_into().unwrap();

    let vdisplay_raw = addressable["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Addressable Y size") as u16;

    let vdisplay = vdisplay_raw.try_into().unwrap();

    let blanking = desc["Blanking"]
        .as_object()
        .expect("Couldn't decode Descriptor Blanking section");

    let hblank_raw = blanking["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Blanking X size") as u16;

    let hblank = hblank_raw.try_into().unwrap();

    let vblank_raw = blanking["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Blanking y size") as u16;

    let vblank = vblank_raw.try_into().unwrap();

    let border = desc["Border"]
        .as_object()
        .expect("Couldn't decode Descriptor Blanking section");

    let hborder_raw = border["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Blanking X size") as u8;

    let hborder = hborder_raw.try_into().unwrap();

    let vborder_raw = border["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Blanking y size") as u8;

    let vborder = vborder_raw.try_into().unwrap();

    let fp = desc["Front porch"]
        .as_object()
        .expect("Couldn't decode Descriptor Front Porch section");

    let hfp_raw = fp["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Front Porch X size") as u16;

    let hfp = hfp_raw.try_into().unwrap();

    let vfp_raw = fp["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Front Porch y size") as u8;

    let vfp = vfp_raw.try_into().unwrap();

    let sync = desc["Sync pulse"]
        .as_object()
        .expect("Couldn't decode Descriptor Sync Pulse section");

    let hsync_raw = sync["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Sync X size") as u16;

    let hsync = hsync_raw.try_into().unwrap();

    let vsync_raw = sync["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Sync y size") as u8;

    let vsync = vsync_raw.try_into().unwrap();

    let size = desc["Image size (mm)"]
        .as_object()
        .expect("Couldn't decode Descriptor Sync Pulse section");

    let hsize_raw = size["x"]
        .as_u64()
        .expect("Couldn't decode Descriptor Size X size") as u16;

    let hsize = hsize_raw.try_into().unwrap();

    let vsize_raw = size["y"]
        .as_u64()
        .expect("Couldn't decode Descriptor Size y size") as u16;

    let vsize = vsize_raw.try_into().unwrap();

    let interlace = desc["Interlace"]
        .as_bool()
        .expect("Couldn't decode Descriptor interlace");

    let pixel_clock_mhz = desc["Pixel clock (MHz)"]
        .as_f64()
        .expect("Couldn't decode Descriptor Pixel Clock");

    let pixel_clock = (pixel_clock_mhz * 1000.0)
        .round()
        .to_u32()
        .unwrap()
        .try_into()
        .unwrap();

    let stereo_str = desc["Stereo viewing"]
        .as_str()
        .expect("Couldn't decode Descriptor stereo");
    let stereo = match stereo_str {
        "No stereo" => EdidDetailedTimingStereo::None,
        "Field sequential stereo, right image when stereo sync signal = 1" => {
            EdidDetailedTimingStereo::FieldSequentialRightOnSync
        }
        "2-way interleaved stereo, right image on even lines" => {
            EdidDetailedTimingStereo::TwoWayInterleavedRightOnEven
        }
        "2-way interleaved stereo, left image on even lines" => {
            EdidDetailedTimingStereo::TwoWayInterleavedLeftOnEven
        }
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

            let analog_sync = EdidDetailedTimingAnalogSync::Composite(serrations, sync_on_rgb);
            EdidDetailedTimingSync::Analog(analog_sync)
        }
        "Digital Composite Sync" => {
            let serrations = sync_type["Serrations"]
                .as_bool()
                .expect("Couldn't decode Sync on RGB");

            let hpol_str = sync_type["Horizontal sync (outside of V-sync)"]
                .as_str()
                .expect("Couldn't decode Descriptor Horizontal Sync polarity");
            let hpol = match hpol_str {
                "Negative" => false,
                "Positive" => true,
                _ => panic!("Couldn't decode Descriptor Vertical Sync polarity"),
            };

            EdidDetailedTimingSync::Digital(
                EdidDetailedTimingDigitalSync::builder()
                    .kind(EdidDetailedTimingDigitalSyncKind::Composite(
                        EdidDetailedTimingDigitalCompositeSync::builder()
                            .serrations(serrations)
                            .build(),
                    ))
                    .hsync_positive(hpol)
                    .build(),
            )
        }
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

            EdidDetailedTimingSync::Digital(
                EdidDetailedTimingDigitalSync::builder()
                    .kind(EdidDetailedTimingDigitalSyncKind::Separate(
                        EdidDetailedTimingDigitalSeparateSync::builder()
                            .vsync_positive(vpol)
                            .build(),
                    ))
                    .hsync_positive(hpol)
                    .build(),
            )
        }
        _ => panic!("Couldn't Decode Descriptor Sync Type"),
    };

    EdidDescriptorDetailedTiming::builder()
        .interlace(interlace)
        .pixel_clock(pixel_clock)
        .sync_type(sync_type_type)
        .stereo(stereo)
        .horizontal_size(hsize)
        .vertical_size(vsize)
        .horizontal_front_porch(hfp)
        .vertical_front_porch(vfp)
        .horizontal_addressable(hdisplay)
        .vertical_addressable(vdisplay)
        .horizontal_blanking(hblank)
        .vertical_blanking(vblank)
        .horizontal_border(hborder)
        .vertical_border(vborder)
        .horizontal_sync_pulse(hsync)
        .vertical_sync_pulse(vsync)
        .build()
}

fn decode_range_limit_cvt(desc: &Value) -> EdidR4DisplayRangeVideoTimingsCVT {
    let cvt_version_str = desc["CVT Version"]
        .as_str()
        .expect("Couldn't decode CVT Version");

    assert_eq!(cvt_version_str, "1.1");

    let cvt = EdidR4DisplayRangeVideoTimingsCVTR1::builder();

    let max_active = desc["Maximum active pixels"]
        .as_u64()
        .expect("Couldn't decode the Maximum active pixels") as u16;
    let mut cvt = cvt.maximum_active_pixels_per_line(max_active);

    let aspect_ratio_supported = desc["Supported aspect ratios"]
        .as_object()
        .expect("Couldn't decode the supported aspect ratios");

    for (ratio, supported) in aspect_ratio_supported.iter() {
        let supported = supported
            .as_bool()
            .expect("Couldn't decode the ratio value");

        if !supported {
            continue;
        }

        let ratio = match ratio.as_str() {
            "15:9 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_15_9,
            "16:9 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_16_9,
            "16:10 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_16_10,
            "4:3 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_4_3,
            "5:4 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_5_4,
            _ => panic!("Unknown ratio value"),
        };

        cvt = cvt.add_supported_aspect_ratio(ratio);
    }

    let preferred_ratio_str = desc["Preferred aspect ratio"]
        .as_str()
        .expect("Couldn't decode the preferred aspect ratio");

    let preferred_ratio = match preferred_ratio_str {
        "15:9 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_15_9,
        "16:9 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_16_9,
        "16:10 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_16_10,
        "4:3 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_4_3,
        "5:4 AR" => EdidR4DisplayRangeVideoTimingsAspectRatio::Ratio_5_4,
        _ => panic!("Unknown ratio value"),
    };
    let cvt = cvt.preferred_aspect_ratio(preferred_ratio);

    let preferred_refresh_rate_raw: u8 = desc["Preferred vertical refresh (Hz)"]
        .as_u64()
        .expect("Couldn't decode the preferred refresh rate")
        .try_into()
        .unwrap();
    let preferred_refresh_rate: EdidDisplayRangeVerticalFreq =
        preferred_refresh_rate_raw.try_into().unwrap();
    let cvt = cvt.preferred_vertical_refresh_rate(preferred_refresh_rate);

    let blanking = desc["CVT blanking support"]
        .as_object()
        .expect("Couldn't decode the CVT blanking support section");

    let reduced_blanking = blanking["Reduced CVT Blanking"]
        .as_bool()
        .expect("Couldn't decode Reduced Blanking");
    let cvt = cvt.reduced_cvt_blanking_supported(reduced_blanking);

    let standard_blanking = blanking["Standard CVT Blanking"]
        .as_bool()
        .expect("Couldn't decode Standard Blanking");
    let cvt = cvt.standard_cvt_blanking_supported(standard_blanking);

    let scaling = desc["Display scaling support"]
        .as_object()
        .expect("Couldn't decode the Display Scaling section");

    let hshrink = scaling["Horizontal Shrink"]
        .as_bool()
        .expect("Couldn't decode Horizontal Shrink");
    let cvt = cvt.horizontal_shrink_supported(hshrink);

    let vshrink = scaling["Vertical Shrink"]
        .as_bool()
        .expect("Couldn't decode Vertical Shrink");
    let cvt = cvt.vertical_shrink_supported(vshrink);

    let hstretch = scaling["Horizontal Stretch"]
        .as_bool()
        .expect("Couldn't decode Horizontal Stretch");
    let cvt = cvt.horizontal_stretch_supported(hstretch);

    let vstretch = scaling["Vertical Stretch"]
        .as_bool()
        .expect("Couldn't decode Vertical Stretch");
    let cvt = cvt.vertical_stretch_supported(vstretch);

    EdidR4DisplayRangeVideoTimingsCVT::R1(cvt.build())
}

fn decode_secondary_gtf_release_3(desc: &Value) -> EdidDisplayRangeVideoTimingsGTF {
    let horizontal_start_frequency_str = desc["Start break frequency"]
        .as_str()
        .expect("Couldn't decode GTF Start Break frequency");

    let horizontal_start_frequency: EdidDisplayRangeVideoTimingsGTFStartFrequency =
        Frequency::from_str(horizontal_start_frequency_str)
            .unwrap()
            .get::<kilohertz>()
            .to_u16()
            .unwrap()
            .try_into()
            .unwrap();

    let c: u8 = desc["C"]
        .as_u64()
        .expect("Couldn't decode GTF Blanking Offset")
        .try_into()
        .unwrap();

    let m: u16 = desc["M"]
        .as_u64()
        .expect("Couldn't decode GTF Blanking Gradient")
        .try_into()
        .unwrap();

    let k: u8 = desc["K"]
        .as_u64()
        .expect("Couldn't decode GTF Blanking Scaling Factor")
        .try_into()
        .unwrap();

    let j: u8 = desc["J"]
        .as_u64()
        .expect("Couldn't decode GTF Blanking Scaling Factor Weighting")
        .try_into()
        .unwrap();

    EdidDisplayRangeVideoTimingsGTF::builder()
        .horizontal_start_frequency(horizontal_start_frequency)
        .blanking_gradient(m)
        .blanking_offset(c)
        .blanking_scaling_factor(k)
        .blanking_scaling_factor_weighting(j)
        .build()
}

fn decode_display_range_release_3(desc: &Value) -> EdidR3DisplayRangeLimits {
    let hrate = desc["Horizontal rate (kHz)"]
        .as_object()
        .expect("Couldn't decode Display Range Horizontal section");

    let hrate_min_raw: u8 = hrate["Minimum"]
        .as_u64()
        .expect("Couldn't decode Display Range Minimum Horizontal frequency")
        .try_into()
        .unwrap();
    let hrate_min: EdidDisplayRangeHorizontalFreq = hrate_min_raw.try_into().unwrap();

    let hrate_max_raw: u8 = hrate["Maximum"]
        .as_u64()
        .expect("Couldn't decode Display Range Maximum Horizontal frequency")
        .try_into()
        .unwrap();
    let hrate_max: EdidDisplayRangeHorizontalFreq = hrate_max_raw.try_into().unwrap();

    let vrate = desc["Vertical rate (Hz)"]
        .as_object()
        .expect("Couldn't decode Display Range Vertical section");

    let vrate_min_raw: u8 = vrate["Minimum"]
        .as_u64()
        .expect("Couldn't decode Display Range Minimum Vertical frequency")
        .try_into()
        .unwrap();
    let vrate_min: EdidDisplayRangeVerticalFreq = vrate_min_raw.try_into().unwrap();

    let vrate_max_raw: u8 = vrate["Maximum"]
        .as_u64()
        .expect("Couldn't decode Display Range Maximum Vertical frequency")
        .try_into()
        .unwrap();
    let vrate_max: EdidDisplayRangeVerticalFreq = vrate_max_raw.try_into().unwrap();

    let pixel_clock_raw: u16 = desc["Pixel clock (MHz)"]
        .as_f64()
        .expect("Couldn't decode the Display Range Maximum Pixel Frequency")
        .round() as u16;
    let pixel_clock: EdidDisplayRangePixelClock = pixel_clock_raw.try_into().unwrap();

    let subtype_str = desc["Subtype"]
        .as_str()
        .expect("Couldn't decode the Display Range Subtype");

    let timings_support = match subtype_str {
        "Default GTF supported" => EdidR3DisplayRangeVideoTimingsSupport::DefaultGTF,
        "Secondary GTF supported - requires default too" => {
            EdidR3DisplayRangeVideoTimingsSupport::SecondaryGTF(decode_secondary_gtf_release_3(
                desc,
            ))
        }
        _ => panic!("Couldn't decode the Display Range Subtype"),
    };

    EdidR3DisplayRangeLimits::builder()
        .min_hfreq(hrate_min)
        .max_hfreq(hrate_max)
        .min_vfreq(vrate_min)
        .max_vfreq(vrate_max)
        .max_pixelclock(pixel_clock)
        .timings_support(timings_support)
        .build()
}

fn decode_display_range_release_4(desc: &Value) -> EdidR4DisplayRangeLimits {
    let hrate = desc["Horizontal rate (kHz)"]
        .as_object()
        .expect("Couldn't decode Display Range Horizontal section");

    let hrate_min_raw: u16 = hrate["Minimum"]
        .as_u64()
        .expect("Couldn't decode Display Range Minimum Horizontal frequency")
        .try_into()
        .unwrap();
    let hrate_min: EdidR4DisplayRangeHorizontalFreq = hrate_min_raw.try_into().unwrap();

    let hrate_max_raw: u16 = hrate["Maximum"]
        .as_u64()
        .expect("Couldn't decode Display Range Maximum Horizontal frequency")
        .try_into()
        .unwrap();
    let hrate_max: EdidR4DisplayRangeHorizontalFreq = hrate_max_raw.try_into().unwrap();

    let vrate = desc["Vertical rate (Hz)"]
        .as_object()
        .expect("Couldn't decode Display Range Vertical section");

    let vrate_min_raw: u16 = vrate["Minimum"]
        .as_u64()
        .expect("Couldn't decode Display Range Minimum Vertical frequency")
        .try_into()
        .unwrap();
    let vrate_min: EdidR4DisplayRangeVerticalFreq = vrate_min_raw.try_into().unwrap();

    let vrate_max_raw: u16 = vrate["Maximum"]
        .as_u64()
        .expect("Couldn't decode Display Range Maximum Vertical frequency")
        .try_into()
        .unwrap();
    let vrate_max: EdidR4DisplayRangeVerticalFreq = vrate_max_raw.try_into().unwrap();

    let pixel_clock_raw = desc["Pixel clock (MHz)"]
        .as_f64()
        .expect("Couldn't decode the Display Range Maximum Pixel Frequency")
        .round() as u16;
    let pixel_clock: EdidDisplayRangePixelClock = pixel_clock_raw.try_into().unwrap();

    let subtype_str = desc["Subtype"]
        .as_str()
        .expect("Couldn't decode the Display Range Subtype");

    let timings_support = match subtype_str {
        "CVT supported" => {
            EdidR4DisplayRangeVideoTimingsSupport::CVTSupported(decode_range_limit_cvt(desc))
        }
        "Default GTF supported" => EdidR4DisplayRangeVideoTimingsSupport::DefaultGTF,
        "Range Limits Only - no additional info" => {
            EdidR4DisplayRangeVideoTimingsSupport::RangeLimitsOnly
        }
        _ => panic!("Couldn't decode the Display Range Subtype"),
    };

    EdidR4DisplayRangeLimits::builder()
        .min_hfreq(hrate_min)
        .max_hfreq(hrate_max)
        .min_vfreq(vrate_min)
        .max_vfreq(vrate_max)
        .max_pixelclock(pixel_clock)
        .timings_support(timings_support)
        .build()
}

fn decode_descriptor_established_timings(desc: &Value) -> EdidR4DescriptorEstablishedTimings {
    let map = desc["Established Timings"]
        .as_object()
        .expect("Couldn't decode the established timings section");

    let mut timings = EdidR4DescriptorEstablishedTimings::builder();
    for (timing, timing_val) in map.iter() {
        let supported = timing_val
            .as_bool()
            .expect("Couldn't decode the timing value");

        if !supported {
            continue;
        }

        let et = match timing.as_str() {
            "1024 x 768 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1024_768_85Hz,
            "1152 x 864 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1152_864_75Hz,
            "1280 x 1024 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_1024_60Hz,
            "1280 x 1024 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_1024_85Hz,
            "1280 x 768 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_768_60Hz,
            "1280 x 768 @ 60 Hz (RB)" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_768_60Hz_RB,
            "1280 x 768 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_768_75Hz,
            "1280 x 768 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_768_85Hz,
            "1280 x 960 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_960_60Hz,
            "1280 x 960 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1280_960_85Hz,
            "1360 x 768 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1360_768_60Hz,
            "1400 x 1050 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_60Hz,
            "1400 x 1050 @ 60 Hz (RB)" => {
                EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_60Hz_RB
            }
            "1400 x 1050 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_75Hz,
            "1400 x 1050 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1400_1050_85Hz,
            "1440 x 900 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1440_900_60Hz,
            "1440 x 900 @ 60 Hz (RB)" => EdidR4DescriptorEstablishedTimingsIII::ET_1440_900_60Hz_RB,
            "1440 x 900 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1440_900_75Hz,
            "1440 x 900 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1440_900_85Hz,
            "1600 x 1200 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_60Hz,
            "1600 x 1200 @ 65 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_65Hz,
            "1600 x 1200 @ 70 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_70Hz,
            "1600 x 1200 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_75Hz,
            "1600 x 1200 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1600_1200_85Hz,
            "1680 x 1050 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1680_1050_60Hz,
            "1680 x 1050 @ 60 Hz (RB)" => {
                EdidR4DescriptorEstablishedTimingsIII::ET_1680_1050_60Hz_RB
            }
            "1680 x 1050 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1680_1050_75Hz,
            "1680 x 1050 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1680_1050_85Hz,
            "1792 x 1344 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1792_1344_60Hz,
            "1792 x 1344 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1792_1344_75Hz,
            "1856 x 1392 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1856_1392_60Hz,
            "1856 x 1392 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1856_1392_75Hz,
            "1920 x 1200 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1920_1200_60Hz,
            "1920 x 1200 @ 60 Hz (RB)" => {
                EdidR4DescriptorEstablishedTimingsIII::ET_1920_1200_60Hz_RB
            }
            "1920 x 1200 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1920_1200_75Hz,
            "1920 x 1200 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1920_1200_85Hz,
            "1920 x 1440 @ 60 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1920_1440_60Hz,
            "1920 x 1440 @ 75 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_1920_1440_75Hz,
            "640 x 350 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_640_350_85Hz,
            "640 x 400 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_640_400_85Hz,
            "640 x 480 @ 85 Hz" => EdidR4DescriptorEstablishedTimingsIII::ET_640_480_85Hz,
            _ => panic!("Couldn't decode the Establish Timing"),
        };

        timings = timings.add_established_timing(et);
    }

    timings.build()
}

fn decode_data_string(desc: &Value) -> EdidDescriptorString {
    let string = desc["Data string"]
        .as_str()
        .expect("Couldn't decode Product Name")
        .to_string();

    EdidDescriptorString::from_str_encoding_unchecked(&string)
}

fn decode_descriptor_name(desc: &Value) -> EdidDescriptorString {
    let name = desc["Data string"]
        .as_str()
        .expect("Couldn't decode Product Name")
        .to_string();

    EdidDescriptorString::from_str_encoding_unchecked(&name)
}

fn decode_descriptor_serial(desc: &Value) -> EdidDescriptorString {
    let serial = desc["Data string"]
        .as_str()
        .expect("Couldn't decode Product Name")
        .to_string();

    EdidDescriptorString::from_str_encoding_unchecked(&serial)
}

fn decode_custom_descriptor(desc: &Value) -> EdidDescriptorCustom {
    let tag = desc["Tag"]
        .as_u64()
        .expect("Couldn't decode the custom descriptor's tag") as u8;

    let data = desc["Blob"]
        .as_array()
        .expect("Couldn't decode Product Name")
        .into_iter()
        .map(|val| val.as_u64().expect("Couldn't decode blob") as u8)
        .collect();

    (tag, data).try_into().unwrap()
}

fn decode_descriptors_release_3(descriptors: &Value) -> Vec<EdidR3Descriptor> {
    let mut descs = Vec::new();

    let list = descriptors
        .as_array()
        .expect("Couldn't decode Descriptors list");

    for desc in list {
        let desc_type = desc["Type"]
            .as_str()
            .expect("Couldn't decode descriptor's type");

        let desc: EdidR3Descriptor = match desc_type {
            "Alphanumeric Data String (ASCII)" => {
                EdidR3Descriptor::DataString(decode_data_string(desc))
            }
            "Detailed Timing Descriptor" => {
                EdidR3Descriptor::DetailedTiming(decode_descriptor_dtd(desc))
            }
            "Display Range Limits Descriptor" => {
                EdidR3Descriptor::DisplayRangeLimits(decode_display_range_release_3(desc))
            }
            "Display Product Name" => EdidR3Descriptor::ProductName(decode_descriptor_name(desc)),
            "Display Product Serial Number" => {
                EdidR3Descriptor::ProductSerialNumber(decode_descriptor_serial(desc))
            }
            "Dummy descriptor" => EdidR3Descriptor::Dummy,
            "Manufacturer Specified Display Descriptor" => {
                EdidR3Descriptor::Custom(decode_custom_descriptor(desc).into())
            }
            _ => panic!("Couldn't decode the descriptor's type: {}", desc_type),
        };

        descs.push(desc);
    }

    descs
}

fn decode_descriptors_release_4(descriptors: &Value) -> Vec<EdidR4Descriptor> {
    let mut descs = Vec::new();

    let list = descriptors
        .as_array()
        .expect("Couldn't decode Descriptors list");

    for desc in list {
        let desc_type = desc["Type"]
            .as_str()
            .expect("Couldn't decode descriptor's type");

        let desc: EdidR4Descriptor = match desc_type {
            "Alphanumeric Data String (ASCII)" => {
                EdidR4Descriptor::DataString(decode_data_string(desc))
            }
            "Detailed Timing Descriptor" => {
                EdidR4Descriptor::DetailedTiming(decode_descriptor_dtd(desc))
            }
            "Display Range Limits Descriptor" => {
                EdidR4Descriptor::DisplayRangeLimits(decode_display_range_release_4(desc))
            }
            "Display Product Name" => EdidR4Descriptor::ProductName(decode_descriptor_name(desc)),
            "Display Product Serial Number" => {
                EdidR4Descriptor::ProductSerialNumber(decode_descriptor_serial(desc))
            }
            "Dummy descriptor" => EdidR4Descriptor::Dummy,
            "Established Timings III" => {
                EdidR4Descriptor::EstablishedTimings(decode_descriptor_established_timings(desc))
            }
            "Manufacturer Specified Display Descriptor" => {
                EdidR4Descriptor::Custom(decode_custom_descriptor(desc).into())
            }
            _ => panic!("Couldn't decode the descriptor's type: {}", desc_type),
        };

        descs.push(desc);
    }

    descs
}

fn decode_and_check_edid_release_3(json: &Value, expected: &[u8]) {
    let base = &json["Base"];

    let manufacturer_info = &base["Manufacturer Info"];
    let edid = EdidRelease3::builder()
        .manufacturer(decode_manufacturer_name(manufacturer_info))
        .product_code(decode_product_code(manufacturer_info))
        .serial_number(decode_serial_number(manufacturer_info))
        .date(decode_date_release_3(manufacturer_info));

    let basic_display = &base["Basic Display"];
    let chromaticity = &base["Chromaticity"];
    let edid = edid
        .display_parameters_features(decode_basic_display_release_3(basic_display))
        .filter_chromaticity(decode_basic_display_chromaticity(
            basic_display,
            chromaticity,
        ));

    let established_timings = &base["Established Timing"];
    let edid = edid.established_timings(decode_established_timings(established_timings));

    let standard_timings = &base["Standard Timing"];
    let edid = edid.standard_timings(decode_standard_timings(standard_timings));

    let descriptors = &base["Descriptors"];
    let edid = edid.descriptors(decode_descriptors_release_3(descriptors));

    let bytes = edid.build().into_bytes();

    assert!(edid_equals(&bytes, &expected));
}

fn decode_and_check_edid_release_4(json: &Value, expected: &[u8]) {
    let base = &json["Base"];

    let manufacturer_info = &base["Manufacturer Info"];
    let edid = EdidRelease4::builder()
        .manufacturer(decode_manufacturer_name(manufacturer_info))
        .product_code(decode_product_code(manufacturer_info))
        .serial_number(decode_serial_number(manufacturer_info))
        .date(decode_date_release_4(manufacturer_info));

    let basic_display = &base["Basic Display"];
    let chromaticity = &base["Chromaticity"];
    let edid = edid
        .display_parameters_features(decode_basic_display_release_4(basic_display))
        .filter_chromaticity(decode_basic_display_chromaticity(
            basic_display,
            chromaticity,
        ));

    let established_timings = &base["Established Timing"];
    let edid = edid.established_timings(decode_established_timings(established_timings));

    let standard_timings = &base["Standard Timing"];
    let edid = edid.standard_timings(decode_standard_timings(standard_timings));

    let descriptors = &base["Descriptors"];
    let edid = edid.descriptors(decode_descriptors_release_4(descriptors));

    let bytes = edid.build().into_bytes();

    assert!(edid_equals(&bytes, &expected));
}

fn decode_and_check_edid(json: &Value, expected: &[u8]) {
    let version = json["Version"].as_str().unwrap();

    match version {
        "1.3" => decode_and_check_edid_release_3(json, expected),
        "1.4" => decode_and_check_edid_release_4(json, expected),
        _ => todo!(),
    }
}

fn compare_unordered_slots<const CHUNK: usize, const LEN: usize>(
    val: &[u8; LEN],
    expected: &[u8; LEN],
) -> bool {
    assert!((LEN % CHUNK) == 0);

    let mut val_chunks: Vec<_> = val.chunks_exact(CHUNK).collect();
    val_chunks.sort();

    let mut expected_chunks: Vec<_> = expected.chunks_exact(CHUNK).collect();
    expected_chunks.sort();

    val_chunks == expected_chunks
}

fn edid_equals(current: &[u8], expected: &[u8]) -> bool {
    let mut checksum_offset = [0; 2];

    if current.len() != expected.len() {
        println!(
            "Sizes don't match: current {} vs expected {}",
            current.len(),
            expected.len()
        );

        return false;
    }

    for i in 0..expected.len() {
        if current[i] == expected[i] {
            continue;
        }

        // The lower bit of Stereo Viewing Support in a Detailed Timing (Bit 0 of Byte 17) can be
        // set either to 0 or 1. Most of the EDIDs in the wild will set it to 0, and that's what
        // we do too but some set it to 1, so we need to consider both equivalents.
        if (i > 0x36) && ((i - 0x36) % 18) == 17 {
            let diff = (current[i] ^ expected[i]) & 0x61;

            if diff == 1 {
                checksum_offset[0] += current[i] - diff;
                checksum_offset[1] += expected[i] - diff;
                continue;
            }
        }

        // The standard timings do not have to be packed at the beginning but can be in any order.
        // Make sure the list of the stantard timings is equal no matter the order.
        if compare_unordered_slots::<2, 16>(
            &current[0x26..0x36].try_into().unwrap(),
            &expected[0x26..0x36].try_into().unwrap(),
        ) {
            continue;
        }

        if i == 0x7f {
            let adjusted_val1 = current[i].wrapping_sub(checksum_offset[1]);
            let adjusted_val2 = expected[i].wrapping_sub(checksum_offset[0]);

            if adjusted_val1 == adjusted_val2 {
                continue;
            }
        }

        println!(
            "Index {:#x} is different: {:#x} vs {:#x}",
            i, current[i], expected[i]
        );
        return false;
    }

    true
}

fn test_edid(edid: &str) {
    let output = Command::new("tests/tools/edid-chamelium/edid2json.py")
        .arg(edid)
        .output()
        .expect("Couldn't decode the EDID");

    assert!(output.status.success());

    let mut input_file = File::open(edid).unwrap();
    let mut input_data: [u8; 0x80] = [0; 0x80];
    input_file.read_exact(&mut input_data).unwrap();

    let output_str =
        std::str::from_utf8(&output.stdout).expect("Couldn't convert the output to UTF-8");

    let json: Value = serde_json::from_str(output_str).expect("Couldn't parse the JSON output");

    decode_and_check_edid(&json, &input_data);
}

#[test_resources("tests/edid-db/edid.tv/*.bin")]
fn test_edidtv(edid: &str) {
    test_edid(edid)
}

#[test_resources("tests/edid-db/linuxhw/*.bin")]
fn test_linuxhw(edid: &str) {
    test_edid(edid)
}
