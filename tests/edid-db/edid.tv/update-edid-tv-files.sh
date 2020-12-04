#! /bin/sh

MAX_ID=1000

for id in $(seq 1 $MAX_ID); do
	file="edid.tv-$id.bin"
	if ls $file* > /dev/null 2>&1; then
		continue
	fi

	echo "$id not found, downloading..."
	curl -sfL "http://edid.tv/edid/$id/download/" -o $file
	if [ $? -ne 0 ]; then
		echo "Couldn't download id $id, stopping."
		exit $?
	fi

	decode=$(cat $file | edid-decode)
	ver=$(echo "$decode" | grep "EDID Structure Version & Revision" | cut -d ':' -f2 | sed 's/^ *//g')
	if [ $ver != "1.4" ]; then
		mv $file $file.disabled-1.3
		continue
	fi

	json=$(../tools/edid-chamelium/edid2json.py $file)
	if [ $? -ne 0 ]; then
		mv $file $file.disabled-json-fails
		continue
	fi

	sleep 1
done
