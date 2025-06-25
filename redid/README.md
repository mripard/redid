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
    - [x] Audio Data Block
      - [x] LPCM
      - [ ] AC-3
      - [ ] MPEG-1
      - [ ] MP3
      - [ ] MPEG-2
      - [ ] AAC LC
      - [ ] DTS
      - [ ] ATRAC
      - [ ] One Bit Audio
      - [ ] Enhanced AC-3
      - [ ] DTS-HD
      - [ ] MAT
      - [ ] DST
      - [ ] WMA Pro
      - [ ] Extension Types
        - [ ] MPEG-4 HE AAC
        - [ ] MPEG-4 HE AAC v2
        - [ ] MPEG-4 AAC LC
        - [ ] DRA
        - [ ] MPEG-4 HE AAC + MPEG Surround
        - [ ] MPEG-4 AAC LC + MPEG Surround
        - [ ] MPEG-H 3D Audio
        - [ ] AC-4
        - [ ] L-PCM 3D Audio
    - [x] Video Data Block
    - [x] Vendor Specific Data Block
      - [ ] HDMI Vendor Specific Data Block (HDMI VSDB)
        - [x] Physical Address
        - [x] DVI Dual
        - [x] Deep Color
        - [x] ACP & ISRC
        - [x] Max TMDS Clock
        - [ ] Content Type
        - [ ] Latency
        - [x] VICs
        - [ ] Image Size
        - [ ] 3D
      - [ ] HDMI Forum Vendor Specific Data Block (HF-VSDB)
    - [x] Speaker Allocation Data Block
    - [ ] VESA Display Transfer Characteristics Data Block
    - [ ] Extended Data Blocks
      - [x] Video Capability Data Block
      - [ ] Vendor-Specific Video Data Block
      - [ ] VESA Display Device Data Block
      - [ ] VESA Video Timing Block Extension
      - [x] Colorimetry Data Block
      - [ ] HDR Static Metadata Data Block
      - [ ] HDR Dynamic Metadata Data Block
      - [ ] Video Format Preference Data Block
      - [ ] YCbCr 4:2:0 Video Data Block
      - [ ] YCbCr 4:2:0 Capability Map Data Block
      - [ ] Vendor-Specific Audio Data Block
      - [ ] Room Configuration Data Block
      - [ ] Speaker Location Data Block
      - [ ] `InfoFrame` Data Block

### Type Safety

- [ ] Main Block

  - [x] Manufacturer Name
    - [x] Mandatory
    - [x] 3 Characters
    - [x] ASCII Only
    - [x] Upper Case Only

  - [x] Product Code
    - [x] Mandatory
    - [x] 2 bytes

  - [x] Serial Number
    - [x] 4 bytes

  - [x] Date
    - [x] Mandatory
    - [x] Year is higher than or equal to 1990
    - [x] EDID 1.3
      - [x] Week is between 1 and 53
    - [x] EDID 1.4
      - [x] Week is between 1 and 54

  - [ ] Basic Display Parameters and Features
    - [x] Mandatory
    - [ ] Video Input Definition
      - [x] Mandatory
        - [ ] Analog
          - [ ] Serrations on the vsync pulse are required when composite sync or sync on green
    - [x] Display Size
      - [x] Mandatory
      - [x] 0x00, 0x00 is allowed for undefined size
    - [x] Gamma
      - [x] Mandatory
    - [ ] Feature Support
      - [x] Mandatory
      - [ ] Bits 3 and 4 must be consistent with bit 7 at 0x14
      - [ ] SRGB
        - [ ] If set, the color information in the Chromaticity Coordinates must match
        - [ ] Signaled, but the Gamma isn't set to 2.2
        - [ ] Not signaled, but the Chromaticities match SRGB
      - [x] EDID 1.4
        - [x] Suspend is deprecated
        - [x] Standby is deprecated
      - [x] Preferred Timing
        - [x] EDID 1.3
          - [x] Required

  - [ ] Display XY Chromaticity Coordinates
    - [x] Mandatory
    - [ ] Needs to be consistent with the Display Color Type in the display parameters

  - [x] Established Timings
    - [x] Required if Plug & Play (by assuming the device is Plug & Play)
    - [x] 640x480@60Hz is required if Plug & Play (by assuming the device is Plug & Play)

  - [x] Standard Timings
    - [x] Horizontal Addressable Pixels is between 256 and 2288, in increment of 8 pixels
    - [x] Field Refresh Rate is between 60 and 123Hz
    - [x] Unused Timings mut be set to 0x01, 0x01

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
      - [x] Frequency is between 10 and 655,350kHz
      - [x] Horizontal Addressable, Blanking, Vertical Addressable and Blanking are between 0 and 4095 pixels
      - [x] HFP and Hsync are between 0 and 1023 pixels
      - [x] VFP, Vsync are between 0 and 63  lines
      - [x] Hsync / vsync are between 0 and 4095 mm
      - [x] Right / Left and Top / Bottom Borders are between 0 and 255 pixels / lines
      - [x] Can be set to 0 if undefined
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

- [ ] Extensions
  - [ ] CTA 861 Extension
    - [ ] Main Block
      - [ ] YCbCr 4:4:4 and YCbCr 4:2:2 always need to be set together
    - [ ] Data Blocks
      - [ ] Video Data Block
        - [ ] Native-capable VICs are between 1 and 64
        - [ ] Other VICs are between 65 and 127 and between 193 and 255
      - [ ] Audio Data Block
        - [x] Audio Format Code 1 has depth as third byte
        - [ ] Audio Format Codes 2 to 8 has maximum bit rate as third byte
        - [ ] Audio Format Codes 9 to 13 has a format dependant payload
        - [ ] Audio Format Code 14 has a profile as third byte payload
        - [ ] Extended Audio Type code is between 4 and 6 and between 8 and 10
        - [ ] Extended Audio Code from 4 to 6 has 2 frame lengths as third byte payload
        - [ ] Extended Audio Code from 8 to 10 has 3 frame lengths as third byte payload
        - [ ] Extended Audio Code 11 has a format dependent value as third byte payload
        - [ ] Extended Audio Code 12 has a format dependent value as third byte payload
        - [ ] Extended Audio Code 13 has a bit depth as third byte payload
    - [ ] HDMI Vendor Specific Data Block (HDMI VSDB)
      - [ ] Physical Address has 4 bits per component
      - [ ] Max TMDS Clock is optional, and if set must be above 165MHz
      - [ ] If Latency bit is set, then video and audio latencies must be set
      - [ ] If Interleaved Latency bit is set, then interlaced video and audio latencies must be set
      - [ ] If Video bit is set, then Flags, VICs and 3D LEN must be set
      - [ ] Video Latency is either unknown, unsupported or between 0 and 500
      - [ ] Audio Latency is either unknown, unsupported or between 0 and 500
      - [ ] If 3D bit is set, some 2D modes are required, see section 8.3.2
