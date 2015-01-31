; Code for the intro rom.
; The assembler is not included here. I used:
; https://code.google.com/p/chasm-asm/

START:
  CLS
  CALL PRINT
  JP END

WAIT:
  LD VD, 30
  LD DT, VD
  SPIN:
    LD VD, DT
    SE VD, 0
    JP SPIN
  RET

PRINT:
  LD VA, 16 ; VA will be our X position
  LD VB, 8  ; VB our Y

  CALL PRINTCR
  ADD VB, 2
  ADD VA, 10

  ; Print the date stored at the date label
  LD VC, 0   ; VC is our loop counter. We'l count over 4 BCD chars
  NEXT_C:
    CALL WAIT
    LD I, DATE
    ADD I, VC  ; Move to next digit in DATE

    LD V0, [I] ; Load that digit into V0
    LD F, V0   ; overrite [I] with font addr for the digit

    DRW VA, VB, 5

    ADD VA, 5 ; Move X, 5 pix over
    ADD VC, 1 ; Inc loop
    SE VC, 4
    JP NEXT_C

  ; Dramatic pause :P
  CALL WAIT
  CALL WAIT

  LD VA, 13
  LD VB, 17
  CALL PRINT_NAME
  RET

; Got kinda lazy here should roll these into some 'print character' routine.
PRINT_NAME:
  CALL PRINTJ
  ADD VA, 4
  CALL PRINTA
  ADD VA, 5
  CALL PRINTK
  ADD VA, 5
  CALL PRINTE

  ADD VA, 8

  CALL PRINTK
  ADD VA, 5
  CALL PRINTE
  ADD VA, 5
  CALL PRINTR
  ADD VA, 5
  CALL PRINTR
  ADD VA, 5
  RET

PRINTCR:
  LD I, CR
  DRW VA, VB, 8
  RET

PRINTJ:
  LD I, J
  DRW VA, VB, 6
  RET

PRINTA:
  LD I, A
  DRW VA, VB, 6
  RET

PRINTK:
  LD I, K
  DRW VA, VB, 6
  RET

PRINTE:
  LD I, E
  DRW VA, VB, 6
  RET

PRINTR:
  LD I, R
  DRW VA, VB, 6
  RET

END:
  JP END

DATE: ;BCD for easy printing
DB 2
DB 0
DB 1
DB 5

CR:
DB $01111110
DB $10000001
DB $10011101
DB $10100001
DB $10100001
DB $10111101
DB $10000001
DB $01111110

J:
DB $11111000
DB $00100000
DB $00100000
DB $00100000
DB $10100000
DB $11100000

A:
DB $00000000
DB $00000000
DB $01110000
DB $10010000
DB $10010000
DB $11101000

K:
DB $10000000
DB $10010000
DB $10100000
DB $11100000
DB $10010000
DB $10001000

E:
DB $00000000
DB $01100000
DB $10010000
DB $11100000
DB $10000000
DB $01110000

R:
DB $00000000
DB $10110000
DB $11000000
DB $10000000
DB $10000000
DB $10000000

