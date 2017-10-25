Zydis Rust Bindings
===================

Rust language bindings for the [Zydis library](https://github.com/zyantific/zydis), a fast and lightweight x86/x86-64 disassembler.

## Building
Please make sure your system meets [all requirements](https://rust-lang-nursery.github.io/rust-bindgen/requirements.html) to be able to use `bindgen`. Then, just invoke:

```
cargo build
```

Or, probably more common, add a dependency to your `Cargo.toml`:

```toml
[dependencies]
zydis = "0.0.3"
```

## Example
```rust
extern crate zydis;
use zydis::gen::*;
use zydis::*;

static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00u8,
];

fn main() {
    let formatter = Formatter::new(ZYDIS_FORMATTER_STYLE_INTEL).unwrap();
    let decoder = Decoder::new(ZYDIS_MACHINE_MODE_LONG_64, ZYDIS_ADDRESS_WIDTH_64).unwrap();

    for (mut instruction, ip) in decoder.instruction_iterator(CODE, 0) {
        let insn = formatter.format_instruction(&mut instruction, 200, None);
        println!("0x{:016X} {}", ip, insn.unwrap());
    }
}
```

#### Output
```
0x0000000000000001 push rcx
0x0000000000000004 lea eax, [rbp-0x01]
0x0000000000000005 push rax
0x0000000000000008 push qword ptr [rbp+0x0C]
0x000000000000000B push qword ptr [rbp+0x08]
0x0000000000000011 call [0x000000007648A5B1]
0x0000000000000013 test eax, eax
0x0000000000000019 js 0x000000000002DB15
```


### Version Map


| Bindings | Zydis    |
| -------- | -------- |
| v0.0.3   | [v2.0.0-develop@e967510](https://github.com/zyantific/zydis/tree/e967510fb251cf39a3556942b58218a9dcac5554) |
| v0.0.2   | [v2.0.0-alpha2](https://github.com/zyantific/zydis/tree/v2.0.0-alpha2) |
| v0.0.1   | [v2.0.0-develop@4a79d57](https://github.com/zyantific/zydis/tree/4a79d5762ea7f15a5961733cc6d3a7704d3d5206) |
