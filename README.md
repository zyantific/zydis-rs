Zydis Rust Bindings
===================

Rust language bindings for the [Zydis library](https://github.com/zyantific/zydis), a fast and lightweight x86/x86-64 disassembler.

```toml
[dependencies]
zydis = "4.0"
```

## Example
```rust
extern crate zydis;

use zydis::*;

#[rustfmt::skip]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00,
];

fn main() -> zydis::Result {
    let fmt = Formatter::new(FormatterStyle::INTEL)?;
    let dec = Decoder::new(MachineMode::LONG_64, StackWidth::_64)?;

    // 0 is the address for our code.
    for insn_info in dec.decode_all::<VisibleOperands>(CODE, 0) {
        let (ip, _raw_bytes, insn) = insn_info?;
        
        // We use Some(ip) here since we want absolute addressing based on the given
        // instruction pointer. If we wanted relative addressing, we'd use `None` instead.
        println!("0x{:016X} {}", ip, fmt.format(Some(ip), &insn)?);
    }

    Ok(())
}
```

### Output

```text
0x0000000000000000 push rcx
0x0000000000000001 lea eax, [rbp-0x01]
0x0000000000000004 push rax
0x0000000000000005 push [rbp+0x0C]
0x0000000000000008 push [rbp+0x08]
0x000000000000000B call [0x000000007648A5B1]
0x0000000000000011 test eax, eax
0x0000000000000013 js 0x000000000002DB15
```

## Version Map


| Bindings | Zydis                                                                                                      |
|----------|------------------------------------------------------------------------------------------------------------|
| v4.0.0   | [v4.0.0](https://github.com/zyantific/zydis/releases/tag/v4.0.0)                                           |
| v3.0.0   | [v3.0.0](https://github.com/zyantific/zydis/releases/tag/v3.0.0)                                           |
| v0.0.4   | [v2.0.2](https://github.com/zyantific/zydis/tree/v2.0.2)                                                   |
| v0.0.3   | [v2.0.0-develop@e967510](https://github.com/zyantific/zydis/tree/e967510fb251cf39a3556942b58218a9dcac5554) |
| v0.0.2   | [v2.0.0-alpha2](https://github.com/zyantific/zydis/tree/v2.0.0-alpha2)                                     |
| v0.0.1   | [v2.0.0-develop@4a79d57](https://github.com/zyantific/zydis/tree/4a79d5762ea7f15a5961733cc6d3a7704d3d5206) |
