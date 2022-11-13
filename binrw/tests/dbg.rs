#[cfg(feature = "std")]
#[test]
fn dbg() {
    use binrw::{io::Cursor, BinRead};

    #[derive(BinRead, Debug)]
    struct Inner(u32);

    #[allow(dead_code)]
    #[derive(BinRead)]
    #[br(big)]
    struct Test {
        before: u16,
        #[br(dbg)]
        value: u32,
        #[br(dbg)]
        inner: Inner,
    }

    // 🥴
    if let Some("1") = option_env!("BINRW_IN_CHILD_PROC") {
        Test::read(&mut Cursor::new(b"\0\0\0\0\0\x04\0\x0e\xff\xed")).unwrap();
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
                    "[{file}:10 | offset 0x2] value = 0x4\n",
                    "[{file}:10 | offset 0x6] inner = Inner(\n",
                    "    0xeffed,\n",
                    ")\n"
                ),
                file = core::file!()
            )
        );
    }
}
