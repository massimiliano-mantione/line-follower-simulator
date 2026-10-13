gnuplot -p -e "set datafile separator ','; plot 'telemetry.csv' using 1:23 with lines, 'telemetry.csv' using 1:24 with lines"
