# EDID Generation Crate

This crate is aimed at providing a typesafe abstraction to generate EDIDs.

## TODO

### General Features

- [ ] Edid 1.0 Support
- [ ] Edid 1.1 Support
- [ ] Edid 1.2 Support
- [x] Edid 1.3 Support
  - [ ] Color Point Descriptor
  - [ ] Standard Timing Descriptor
- [x] Edid 1.4 Support
  - [ ] Color Point Descriptor
  - [ ] Standard Timing Descriptor
  - [ ] Color Management Descriptor
  - [ ] CVT 3 byte Timing Codes

### Specific Features

- [ ] Extensions
  - [ ] Video Timing Block Extension (VTB-EXT)
  - [ ] Display Information Extension (DI-EXT)
  - [ ] Localized String Extension (LS-EXT)
  - [ ] Digital Packet Video Link Extension (DPVL-EXT)
  - [ ] CEA-861 Series Timing Extensions

### Type Safety

- [x] Manufacturer Name
  - [x] Mandatory
  - [x] 3 Characters
  - [x] ASCII Only
  - [x] Upper Case Only

- [ ] Product Code
  - [ ] Mandatory
  - [ ] 2 bytes

- [ ] Serial Number
  - [ ] 4 bytes

- [ ] Date
  - [ ] Mandatory
  - [ ] Year is higher than or equal to 1990
  - [ ] EDID 1.3
    - [ ] Week is between 1 and 53
  - [ ] EDID 1.4
    - [ ] Week is between 1 and 54

- [ ] Basic Display Parameters and Features
  - [ ] Mandatory
  - [ ] Video Input Definition
    - [ ] Mandatory
      - [ ] Analog
        - [ ] Serrations on the vsync pulse are required when composite sync or sync on green
  - [ ] Display Size
    - [ ] Mandatory
    - [ ] 0x00, 0x00 is allowed for undefined size
  - [ ] Gamma
    - [ ] Mandatory
  - [ ] Feature Support
    - [ ] Mandatory
    - [ ] Bits 3 and 4 must be consistent with bit 7 at 0x14
    - [ ] SRGB
      - [ ] If set, the color information in the Chromaticity Coordinates must match
      - [ ] Signaled, but the Gamma isn't set to 2.2
      - [ ] Not signaled, but the Chromaticities match SRGB
    - [ ] EDID 1.4
      - [ ] Suspend is deprecated
      - [ ] Standby is deprecated
    - [ ] Preferred Timing
      - [ ] EDID 1.3
        - [ ] Required

- [ ] Display XY Chromaticity Coordinates
  - [ ] Mandatory

- [ ] Established Timings
  - [ ] Required if Plug & Play
  - [ ] 640x480@60Hz is required if Plug & Play

- [ ] Standard Timings
  - [ ] Horizontal Addressable Pixels is between 256 and 2288, in increment of 8 pixels
  - [ ] Field Refresh Rate is between 60 and 123Hz
  - [ ] Unused Timings mut be set to 0x01, 0x01

- [ ] Descriptors
  - [ ] Detailed Descriptors must be first and is preferred mode
  - [ ] EDID 1.3
    - [ ] Display Product Name is required
    - [ ] Display Range Limits is required
  - [ ] EDID 1.4
    - [ ] Display Range Limits is required if continuous frequency, recommended otherwise
    - [ ] Display Product Name is recommended
  - [ ] Detailed Timings
    - [ ] Display Size is set (in the base block), but Image size isn't
    - [ ] Display Size is smaller than the image size
    - [ ] Image Size is set but the Display Size isn't
    - [ ] Frequency is between 10 and 655,350kHz
    - [ ] Horizontal Addressable, Blanking and Vertical Addressable are between 0 and 4095 pixels
    - [ ] HFP and Hsync are between 0 and 1023 pixels
    - [ ] VFP, Vsync are between 0 and 63  lines
    - [ ] Hsync / vsync are between 0 and 4095 mm
    - [ ] Right / Left and Top / Bottom Borders are between 0 and 255 pixels / lines
    - [ ] Can be set to 0 if undefined
  - [ ] Display Range Limits
    - [ ] GTF
      - [ ] Minimum Horizontal Rate is higher that Maximum Horizontal Rate
      - [ ] Minimum Vertical Rate is higher than Maximum Vertical Rate
      - [ ] Default GTF
        - [ ] Byte 11 is set to 0x0a
        - [ ] Bytes 12-17 are set to 0x20
      - [ ] Secondary GTF
        - [ ] Byte 11 is set to 0x00
        - [ ] Start Frequency is less than the highest P/N Frequency
      - [ ] EDID 1.4
        - [ ] Vertical Rates between 1 and 510 Hz
        - [ ] Horizontal Rates between 1 and 510kHz
        - [ ] GTF is deprecated in favor of CVT
        - [ ] Video Timing Support
          - [ ] Descriptor required if bit 0 in 0x18 is set
          - [ ] Default GTF bit can be set only if bit 0 in 0x18 is set
          - [ ] Secondary GTF bit can be set only if bit 0 in 0x18 is set
          - [ ] CVT Supported bit can be set only if bit 0 in 0x18 is set
          - [x] Range Limits Only can only bet set for EDID 1.4
          - [x] CVT Supported can only bet set for EDID 1.4
  - [ ] Strings
    - [ ] Is not empty
    - [x] Up to 13 chars
    - [x] ASCII Only
    - [x] End with 0x0a
    - [x] Padded with 0x20
    - [ ] Types
      - [ ] Product Name Descriptor
      - [ ] Product Serial Descriptor
      - [ ] Alphanumeric Data String Descriptor
