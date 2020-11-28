use std::io::Write;

use edid::EDID;
use edid::EDIDVersion;

fn main() {
    let mut stdout = std::io::stdout();
    let edid = EDID::new(EDIDVersion::V1R4)
        .serialize();

    stdout.write_all(&edid).unwrap();
    stdout.flush().unwrap();
}