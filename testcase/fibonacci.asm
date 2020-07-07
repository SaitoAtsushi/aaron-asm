; これはフィボナッチ数を計算するコードです
        save 2, 100  ; 入力値 100
        save 0, 0   ; F0 = 0
        save 1, 1   ; F1 = 1
        save 3, 0
loop    decr 2, finish
        decr 3, label
        incr 1, [0]
        decr -1, loop
label   incr 0, [1]
        incr 3, 1
        decr -1, loop
finish  decr 3, label2
        save 0, [1]
label2  halt
