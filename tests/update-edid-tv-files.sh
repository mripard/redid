#! /bin/sh

BASE_DIR=$(dirname $0)
EDID_DIR=$BASE_DIR/edid-db/edid.tv/

MAX_FAILURES=10
MAX_ID=1000

failures=0
for id in $(seq 1 $MAX_ID); do
	file="$EDID_DIR/edid.tv-$id.bin"
	if ls $file* > /dev/null 2>&1; then
		failures=0
		continue
	fi

	echo "$id not found, downloading..."
	curl -sfL "http://edid.tv/edid/$id/download/" -o $file
	if [ $? -ne 0 ]; then
		failures=$((failures + 1))
		if ((failures >= MAX_FAILURES)); then
			echo "Failed to fetch last $MAX_FAILURES IDs. Stopping."
			exit $?
		fi

		echo "Couldn't download id $id, skipping."
		continue
	fi

	failures=0

	decode=$(cat $file | edid-decode)
	if [ $? -ne 0 ]; then
		mv $file $file.disabled-edid-decode-failed
		continue
	fi

	ver=$(echo "$decode" | grep "EDID Structure Version & Revision" | cut -d ':' -f2 | sed 's/^ *//g')
	if [ $ver != "1.4" ]; then
		mv $file $file.disabled-$ver
		continue
	fi

	json=$($BASE_DIR/tools/edid-chamelium/edid2json.py $file)
	if [ $? -ne 0 ]; then
		mv $file $file.disabled-json-fails
		continue
	fi

	sleep 1
done
