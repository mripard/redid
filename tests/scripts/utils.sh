#! /bin/bash

BASE_DIR=$(dirname $(realpath "$0"))
TOOLS_DIR="$BASE_DIR/../tools"

function check_file() # file path
{
    local file

    file=$1

    echo $BASE_DIR

    if ! edid-decode --check < "$file" > /dev/null 2>&1
    then
        mv "$file" "$file.disabled-edid-decode-failed"
        return 0
    fi

    if ! json=$("$TOOLS_DIR/edid-chamelium/edid2json.py" "$file")
    then
        mv "$file" "$file.disabled-json-fails"
        return 0
    fi

    ver=$(echo "$json" | jq -r '.Version')
    if [[ $ver != "1.3" && $ver != "1.4" ]]; then
        mv "$file" "$file.disabled-$ver"
        return 0
    fi

    ver=$(echo "$json" | jq -r '.Base.Descriptors[] | select(.Type == "Display Range Limits Descriptor" and .Subtype=="CVT supported") | ."CVT Version"')
    if [ -n "$ver" ]; then
        if [ "$ver" != "1.1" ]; then
            mv "$file" "$file.disabled-cvt-$ver"
            return 0
        fi
    fi

    return 0
}