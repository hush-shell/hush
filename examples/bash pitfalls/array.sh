#!/usr/bin/env bash


# Consider a file containing measurements of any given parameter of a machine learning
# model in production. Most values occur in the range 0-10, but some values get quite
# larger. One might want to generate a notification if the average of such values exceeds
# some threshold. That is, the average of the values higher than 10 is higher than X.
data=$(cat <<'END_HEREDOC'
16.47
8.7
8.4
4.22
14.96
71.56
1.73
71.56
2.4
208.09
7.2
0.09
2.6
205.39
1.6
1.0
3.1
END_HEREDOC
)

# In pure bash, this is practically impossible to do. A solution using awk is quite
# feasible, but awk itself is just another programming language like python or perl. With
# awk, one can't easily call external programs, which in more sophisticated scenarios, can
# be required as an intermediate step in the calculation.

echo "$data" \
	| awk '{ if ($1 > 10.0) { sum += $1; count++; } } END { print sum/count }'
