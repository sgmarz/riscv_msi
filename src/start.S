.section .text.init

.global _start
_start:
.option norelax
    la      sp, _stack_end
    la      gp, __global_pointer$

    csrr    a0, mhartid

    # Allocate 2^13 = 8K of stack space
    slli    t0, a0, 13
    sub     sp, sp, t0

    la      t0, trap
    csrw    mtvec, t0
    la      t0, main
    csrw    mepc, t0
    li      t0, (3 << 11) | (1 << 7)
    csrw    mstatus, t0
    li      t0, (1 << 11)
    csrw    mie, t0

    la      ra, park
    mret
.type _start, function
.size _start, . - _start


park:
    wfi
    j       park
.type park, function
.size park, . - park

