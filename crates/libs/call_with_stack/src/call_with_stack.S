.intel_syntax noprefix

.text

.align 16
.globl call_with_stack_raw
call_with_stack_raw:

mov rbp, rsp
mov rsp, rdx

call rsi

mov rsp, rbp

ret
ud2


.align 16
.globl jump_with_stack_raw
jump_with_stack_raw:
# On function entry, the stack has to be aligned to (rsp % 16 == 8)
# Our argument should be a proper stack, which is 16 byte aligned by default
# A normal call would push the return address into this spot, but we have to decrement it manually
sub rdx, 8

mov rsp, rdx
jmp rsi

ud2
