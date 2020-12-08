#!/usr/bin/env bash


# Ceph is a distributed file system, which provides the `ceph df` command for inspecting
# space usage. An example of such command's output follows:
ceph_df_out=$(cat <<'END_HEREDOC'
RAW STORAGE:
    CLASS     SIZE        AVAIL       USED        RAW USED     %RAW USED
    hdd        62 TiB      52 TiB      10 TiB       10 TiB         16.47
    ssd       8.7 TiB     8.4 TiB     370 GiB      377 GiB          4.22
    TOTAL      71 TiB      60 TiB      11 TiB       11 TiB         14.96

POOLS:
    POOL                ID     STORED      OBJECTS     USED        %USED     MAX AVAIL     QUOTA OBJECTS     QUOTA BYTES     DIRTY       USED COMPR     UNDER COMPR
    rbd-kubernetes      36     288 GiB      71.56k     865 GiB      1.73        16 TiB     N/A               N/A              71.56k            0 B             0 B
    rbd-cache           41     2.4 GiB     208.09k     7.2 GiB      0.09       2.6 TiB     N/A               N/A             205.39k            0 B             0 B
    cephfs-metadata     51     529 MiB         221     1.6 GiB         0        16 TiB     N/A               N/A                 221            0 B             0 B
    cephfs-data         52     1.0 GiB         424     3.1 GiB         0        16 TiB     N/A               N/A                 424            0 B             0 B
END_HEREDOC
)
# One might want to have a script check if any of the pools has exceeded some usage
# percentage threshold. That is, for a threshold T, the script should check if there is
# any line where the %USED column exceeds T. One possible implementation of such script
# follows.

threshold=80.0

usages=$(
	echo "$ceph_df_out" \
		| sed '0,/^POOLS:$/d' \
		| tail -n +2 \
		| tr -s ' ' \
		| cut -d ' ' -f9
	# Well, this is not great to start with. The sed(1) line can be a little cryptic, and we
	# have to work around the remaining lime with tail(1).
)

for usage in $usages; do
	# Here comes one of the main issues. We can't work with floats in bash. We must resort
	# to bc(1) or some other alternative. And how do we get the result back? Well, bc(1)
	# outputs `0` or `1` for relational operators. We must then convert such value to
	# boolean, using the numeric context `(( ))`. That's a lot for what should be simply
	# `$usage > $threshold`, and it introduces a whole myriad of possible issues of its own.
	if (( $(echo "$usage > $threshold" | bc) )); then
		# Emit a warning. Maybe we could send an email here.
		echo "Pool usage above threshold!!!"
	fi
	# Furthermore, it doesn't scale. What if we wanted to sum a given column? Things get out
	# of hand pretty quickly.
done
