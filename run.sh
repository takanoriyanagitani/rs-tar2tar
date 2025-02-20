#!/bin/sh

input=./sample.d/input.tar

geninput(){
	echo generating input file...

	mkdir -p sample.d

	echo hw > sample.d/hw1.txt
	echo wl > sample.d/hw2.txt
	touch     sample.d/empty.dat

	ls sample.d/*.{txt,dat} |
		tar \
			--create \
			--verbose \
			--file "${input}" \
			--files-from=-
}

test -f "${input}" || geninput

export ENV_KEEP=false
export ENV_KEEP=true

ENV_SIMPLE_FILTER_PREFIX=sample.d/hw
ENV_SIMPLE_FILTER_PREFIX=sample.d/em

export ENV_SIMPLE_FILTER_SUFFIX=.dat
export ENV_SIMPLE_FILTER_SUFFIX=.txt

cat "${input}" |
	./rs-tar2tar |
	tar \
		--list \
		--verbose
