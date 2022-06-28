.altmacro
.macro savegp i, stor=t6
    sw  x\i, \i * 8(\stor)
.endm

.macro loadgp i, stor=t6
    lw  x\i, \i * 8(\stor)
.endm

.section .text
.global trap
.align 4
trap:
    csrrw   t6, mscratch, t6
    .set i, 1
    .rep 30
        savegp %i
        .set i, i + 1
    .endr
    mv      t5, t6
    csrrw   t6, mscratch, t6
    savegp  31, t5

    call    rust_trap

    csrr    t6, mscratch
    .set i, 1
    .rep 31
        loadgp %i
        .set i, i + 1
    .endr 
    # j .
    mret
.type trap, function
.size trap, . - trap
