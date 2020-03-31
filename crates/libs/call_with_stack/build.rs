fn main() {
    nasm_rs::Build::new()
        .file("src/call_with_stack.asm")
        .compile("call_with_stack_asm");
}
