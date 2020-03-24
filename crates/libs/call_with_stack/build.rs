fn main() {
    cc::Build::new()
        .file("src/call_with_stack.S")
        .compile("call-with-stack");
}
