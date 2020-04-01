section .text

struc register_context
    .rax: resq 1
    .rbx: resq 1
    .rcx: resq 1
    .rdx: resq 1

    .rbp: resq 1

    .rsi: resq 1
    .rdi: resq 1

    .r8: resq 1
    .r9: resq 1
    .r10: resq 1
    .r11: resq 1
    .r12: resq 1
    .r13: resq 1
    .r14: resq 1
    .r15: resq 1
endstruc

align 16
global asm_breakpoint_handler
extern rust_breakpoint_handler
asm_breakpoint_handler:

; Save register context
sub rsp, register_context_size

mov [rsp + register_context.rax], rax
mov [rsp + register_context.rbx], rbx
mov [rsp + register_context.rcx], rcx
mov [rsp + register_context.rdx], rdx

mov [rsp + register_context.rbp], rbp

mov [rsp + register_context.rdi], rdi
mov [rsp + register_context.rsi], rsi

mov [rsp + register_context.r8], r8
mov [rsp + register_context.r9], r9
mov [rsp + register_context.r10], r10
mov [rsp + register_context.r11], r11
mov [rsp + register_context.r12], r12
mov [rsp + register_context.r13], r13
mov [rsp + register_context.r14], r14
mov [rsp + register_context.r15], r15

; Prepare arguments for Rust handler

lea rdi, [rsp + register_context_size]
mov rsi, rsp

; Align stack if necessary
mov rbp, rsp
and rsp, ~15

call rust_breakpoint_handler

mov rsp, rbp

; Restore context

mov rax, [rsp + register_context.rax]
mov rbx, [rsp + register_context.rbx]
mov rcx, [rsp + register_context.rcx]
mov rdx, [rsp + register_context.rdx]

mov rbp, [rsp + register_context.rbp]

mov rdi, [rsp + register_context.rdi]
mov rsi, [rsp + register_context.rsi]

mov r8, [rsp + register_context.r8]
mov r9, [rsp + register_context.r9]
mov r10, [rsp + register_context.r10]
mov r11, [rsp + register_context.r11]
mov r12, [rsp + register_context.r12]
mov r13, [rsp + register_context.r13]
mov r14, [rsp + register_context.r14]
mov r15, [rsp + register_context.r15]

add rsp, register_context_size

; Return
iretq
ud2
