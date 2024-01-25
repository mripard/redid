#! /bin/bash 

BASE_DIR=$(dirname $(realpath "$0"))
EDID_DIR=$BASE_DIR/../edid-db/
LINUXHW_DIR="$EDID_DIR/linuxhw"
EDID_ARCHIVE=linuxhw-edid.zip

source "${BASE_DIR}/utils.sh"

mkdir -p "$EDID_DIR"
mkdir -p "$LINUXHW_DIR"

cd "$EDID_DIR" || exit

if ! curl https://codeload.github.com/linuxhw/EDID/zip/refs/heads/master -o $EDID_ARCHIVE
then
    exit 1
fi

if ! unzip $EDID_ARCHIVE
then
    exit 1
fi

find "EDID-master/Analog" "EDID-master/Digital" -type f -print0 | while read -r -d $'\0' file
do
    echo "Found $file"

    name=$(basename "$file")
    newfile="$LINUXHW_DIR/linuxhw-$name.bin"

    if ! edid-decode -o raw - "$newfile" < "$file"
    then
        continue
    fi

    check_file "$newfile"
done