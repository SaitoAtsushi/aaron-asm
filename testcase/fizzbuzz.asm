              save 1, 100
              decr -1, main
displayFizz   putc 70
              putc 105
              putc 122
              putc 122
              decr -1, [4]
displayBuzz   putc 66
              putc 117
              putc 122
              putc 122
              decr -1, [4]
main          save 3,[2]
              putn [2]
loop1         decr 3, remainder1, 3
              decr -1, loop1
remainder1    decr 3, else1
              decr -1, checkBuzz
else1         save 4, pc
              incr 4, 2
              decr -1, displayFizz
checkBuzz     save 3, [2]
loop2         decr 3, remainder2, 5
              decr -1, loop2
remainder2    decr 3, else2
              decr -1, goNext
else2         save 4, pc
              incr 4, 2
              decr -1, displayBuzz
goNext        incr 2
              putc 10
              decr 1, finish
              decr -1, main
finish        halt
