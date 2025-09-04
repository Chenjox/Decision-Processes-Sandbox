set datafile separator ','


plot "results.csv" u 1:2 w l, "results.csv" u 1:3 w l, "results.csv" u 1:($3 - $2) w l

pause -1