set datafile separator ','


plot "results.csv" u 1:2 w lp, "results.csv" u 1:3 w lp, "results.csv" u 1:($3 - $2) w lp

pause -1