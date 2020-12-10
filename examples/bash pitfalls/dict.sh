#!/usr/bin/env bash


# Consider the scenario where we have a bunch of files, and we want to copy them to some
# servers based on a given property. The associative array below (requires bash >= 4.0)
# describes to which server each kind of file must go.

declare -A target_addresses=(
	# Here we have to hack the remote addresses as a single string of space separated items.
	['100']='10.0.0.1:/data/ 10.0.0.2:/data/ 10.0.0.8:/data/'
	['200']='10.0.0.3:/data/ 10.0.0.4:/data/'
	['300']='10.0.0.5:/data/ 10.0.0.6:/data/'
	['400']='10.0.0.7:/data/ 10.0.0.8:/data/'
	['500']='10.0.0.9:/data/ 10.0.0.10:/data/ 10.0.0.8:/data/'
	['600']='10.0.0.1:/data/ 10.0.0.2:/data/'
	['700']='10.0.0.3:/data/ 10.0.0.4:/data/'
	['800']='10.0.0.5:/data/ 10.0.0.6:/data/'
	['900']='10.0.0.7:/data/ 10.0.0.8:/data/ 10.0.0.8:/data/'
)


for input_file in ./*; do
	key=$(echo "$input_file" | cut -d '_' -f 2) # Extract the key from the input file name.

	# Here, we leverage the shell's standard word splitting on spaces. If any of the target
	# directories' name contains spaces, then this would break. We would then have to use
	# another character as separator, and manipulate the IFS variable accordingly. In fact,
	# there is no character we can use that would work in every single case, as unix
	# filenames may contain everything but the null character. Furthermore, if the paths
	# contain asterisks, glob expansion takes place.
	for remote_address in ${target_addresses[$key]}; do
		echo $input_file $remote_address
	done
done
