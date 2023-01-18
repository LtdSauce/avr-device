#!/bin/bash

pushd /tmp
wget https://packs.download.microchip.com/Microchip.ATmega_DFP.3.0.158.atpack
unzip Microchip.ATmega_DFP.3.0.158.atpack
popd
cp -vf /tmp/atdf/* vendor/
pushd vendor

# the names are all with some upper case letters. Make everything lowercasee
for file in $(ls | grep [A-Z] ); do
	mv $file `echo $file | tr 'A-Z' 'a-z'`
done
for file in *.atdf; do
	device_name="${file%.atdf}"
	pushd ../patch
	if [ ! -f ${device_name}.yaml ]; then
		echo "Patchfile for ${device_name}  does not exists. Creating trivial one"
		echo "_svd: ../svd/${device_name}.svd" > ../patch/${device_name}.yaml
	fi
	popd
done

echo "atdf files updated. Rerun make and everything now"
