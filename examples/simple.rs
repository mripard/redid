use std::io::Write;

use edid::EDIDVersion;
use edid::EDID;

fn main() {
    let mut stdout = std::io::stdout();
    let edid = EDID::new(EDIDVersion::V1R4).serialize();

    stdout.write_all(&edid).unwrap();
    stdout.flush().unwrap();
}
