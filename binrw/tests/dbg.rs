#[cfg(feature = "std")]
#[test]
fn dbg() {
    use binrw::{io::Cursor, BinRead};

    #[allow(dead_code)]
    #[derive(BinRead, Debug)]
    struct Inner(u32);

    #[allow(dead_code)]
    #[derive(BinRead)]
    #[br(big)]
    struct Test {
        before: u16,
        #[br(dbg, pad_before = 2, pad_after = 1)]
        value: u32,
        #[br(dbg, align_before = 10, align_after = 16)]
        inner: Inner,
        #[br(dbg, pad_size_to = 4)]
        last: u8,
        #[br(dbg)]
        terminator: u8,
        #[br(dbg, align_size_to = 2)]
        after: u8,
    }

    // 🥴
    if let Some("1") = option_env!("BINRW_IN_CHILD_PROC") {
        Test::read(&mut Cursor::new(
            b"\0\0\xff\xff\0\0\0\x04\xff\xff\0\x0e\xff\xed\xff\xff\x42\0\0\0\x69\x25\0",
        ))
        .unwrap();
    } else {
        use std::process::{Command, Stdio};

        let result = Command::new(env!("CARGO"))
            .env("BINRW_IN_CHILD_PROC", "1")
            .args(["test", "-q", "--test", "dbg", "--", "--nocapture"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .unwrap()
            .stderr;

        assert_eq!(
            std::str::from_utf8(&result).unwrap(),
            format!(
                concat!(
                    "[{file}:{offset_0} | pad_before 0x2]\n",
                    "[{file}:{offset_0} | offset 0x4] value = 0x4\n",
                    "[{file}:{offset_0} | pad_after 0x1]\n",
                    "[{file}:{offset_1} | align_before 0xa]\n",
                    "[{file}:{offset_1} | offset 0xa] inner = Inner(\n",
                    "    0xeffed,\n",
                    ")\n",
                    "[{file}:{offset_1} | align_after 0x10]\n",
                    "[{file}:{offset_2} | offset 0x10] last = 0x42\n",
                    "[{file}:{offset_2} | pad_size_to 0x4]\n",
                    "[{file}:{offset_3} | offset 0x14] terminator = 0x69\n",
                    "[{file}:{offset_4} | offset 0x15] after = 0x25\n",
                    "[{file}:{offset_4} | align_size_to 0x2]\n",
                ),
                file = core::file!(),
                offset_0 = if cfg!(nightly) { 16 } else { 11 },
                offset_1 = if cfg!(nightly) { 18 } else { 11 },
                offset_2 = if cfg!(nightly) { 20 } else { 11 },
                offset_3 = if cfg!(nightly) { 22 } else { 11 },
                offset_4 = if cfg!(nightly) { 24 } else { 11 },
            )
        );
    }
}
