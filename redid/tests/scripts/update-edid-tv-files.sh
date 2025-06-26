#! /bin/bash

BASE_DIR=$(dirname $(realpath "$0"))
EDID_DIR=$BASE_DIR/../edid-db/
EDIDTV_DIR="$EDID_DIR/edid.tv"

MAX_FAILURES=10
MAX_ID=5000

source "${BASE_DIR}/utils.sh"

mkdir -p "$EDID_DIR"
mkdir -p "$EDIDTV_DIR"

failures=0
for id in $(seq 1 $MAX_ID); do
    file="$EDIDTV_DIR/edid.tv-$id.bin"
    if ls "$file*" >/dev/null 2>&1; then
        failures=0
        continue
    fi

    echo "$id not found, downloading..."
    curl -sfL "http://edid.tv/edid/$id/download/" -o "$file"
    ret=$?
    if [ $ret -ne 0 ]; then
        failures=$((failures + 1))
        if ((failures >= MAX_FAILURES)); then
            echo "Failed to fetch last $MAX_FAILURES IDs. Stopping."
            exit $ret
        fi
        
        echo "Couldn't download id $id, skipping."
        continue
    fi

    failures=0
    check_file "$file"
done
