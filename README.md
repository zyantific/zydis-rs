Zydis Rust Bindings
===================

Rust language bindings for [Zydis](https://github.com/zyantific/zydis), a fast and lightweight x86/x86-64 disassembler library.

```toml
[dependencies]
zydis = "4.0-beta.1"
```

## Example
```rust
use zydis::*;

#[rustfmt::skip]
static CODE: &'static [u8] = &[
    0x51, 0x8D, 0x45, 0xFF, 0x50, 0xFF, 0x75, 0x0C, 0xFF, 0x75, 0x08,
    0xFF, 0x15, 0xA0, 0xA5, 0x48, 0x76, 0x85, 0xC0, 0x0F, 0x88, 0xFC,
    0xDA, 0x02, 0x00,
];

fn main() -> zydis::Result {
    let fmt = Formatter::intel();
    let dec = Decoder::new64()?;

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

Since version 3.0.0 the binding's major and minor versions are tethered to the Zydis version. The binding's patch 
version is independent of the Zydis version and can be bumped for binding-only changes. Every cargo crate release
has a corresponding git tag.

<details>
  <summary>Version map for older releases</summary>

| Bindings | Zydis                                                                                                      |
|----------|------------------------------------------------------------------------------------------------------------|
| v0.0.4   | [v2.0.2](https://github.com/zyantific/zydis/tree/v2.0.2)                                                   |
| v0.0.3   | [v2.0.0-develop@e967510](https://github.com/zyantific/zydis/tree/e967510fb251cf39a3556942b58218a9dcac5554) |
| v0.0.2   | [v2.0.0-alpha2](https://github.com/zyantific/zydis/tree/v2.0.0-alpha2)                                     |
| v0.0.1   | [v2.0.0-develop@4a79d57](https://github.com/zyantific/zydis/tree/4a79d5762ea7f15a5961733cc6d3a7704d3d5206) |

</details>
