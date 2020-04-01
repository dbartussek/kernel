section .text

align 16
global asm_perform_system_call
asm_perform_system_call:
    int 0x80
    ret
