set datafile separator ","

# viridis
load 'viridis.pal'
set palette maxcolors 20

set pm3d map
splot "pitting-results.csv" matrix

pause -1