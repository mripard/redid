use bon::Builder;
#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{
    EdidDescriptorDetailedTiming, EdidDescriptorString, EdidEstablishedTiming, EdidExtension,
    EdidExtensionCTA861, EdidExtensionCTA861Revision3, EdidFilterChromaticity, EdidManufactureDate,
    EdidManufacturer, EdidProductCode, EdidR3BasicDisplayParametersFeatures, EdidR3Descriptor,
    EdidR3DisplayRangeLimits, EdidRelease3, IntoBytes, EDID_BASE_LEN,
};

#[derive(Builder, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct HdmiEdid {
    #[builder(field)]
    #[cfg_attr(feature = "serde", serde(default))]
    descriptors: Vec<EdidR3Descriptor>,

    #[builder(field)]
    #[cfg_attr(feature = "serde", serde(default))]
    extensions: Vec<EdidExtension>,

    manufacturer: EdidManufacturer,
    product_code: EdidProductCode,
    product_name: EdidDescriptorString,
    date: EdidManufactureDate,
    filter_chromaticity: EdidFilterChromaticity,
    display_parameters_features: EdidR3BasicDisplayParametersFeatures,
    limits: EdidR3DisplayRangeLimits,
    preferred_timing: EdidDescriptorDetailedTiming,
    cta861_extension: EdidExtensionCTA861Revision3,
}

impl From<HdmiEdid> for EdidRelease3 {
    fn from(value: HdmiEdid) -> Self {
        let mut base = EdidRelease3::builder()
            .manufacturer(value.manufacturer)
            .product_code(value.product_code)
            .date(value.date)
            .filter_chromaticity(value.filter_chromaticity)
            .display_parameters_features(value.display_parameters_features)
            .preferred_timing(value.preferred_timing)
            .add_established_timing(EdidEstablishedTiming::ET_640_480_60hz)
            .add_descriptor(EdidR3Descriptor::ProductName(value.product_name))
            .add_descriptor(EdidR3Descriptor::DisplayRangeLimits(value.limits))
            .add_extension(EdidExtension::CTA861(EdidExtensionCTA861::Revision3(
                value.cta861_extension,
            )));

        for desc in value.descriptors {
            base = base.add_descriptor(desc);
        }

        for ext in value.extensions {
            base = base.add_extension(ext);
        }

        base.build()
    }
}

impl IntoBytes for HdmiEdid {
    fn into_bytes(self) -> Vec<u8> {
        EdidRelease3::from(self).into_bytes()
    }

    fn size(&self) -> usize {
        EDID_BASE_LEN
    }
}
