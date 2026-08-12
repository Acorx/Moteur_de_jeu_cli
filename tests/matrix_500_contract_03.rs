fn contract(n: usize) {
    let row = include_str!("TEST_MATRIX_500.md")
        .lines()
        .filter(|l| {
            l.starts_with('|')
                && l.split('|')
                    .nth(1)
                    .is_some_and(|x| x.trim().as_bytes().last().is_some_and(u8::is_ascii_digit))
        })
        .nth(n - 1)
        .unwrap();
    let c: Vec<_> = row.split('|').map(str::trim).collect();
    let json = serde_json::json!({"id":c[1],"category":c[2],"objective":c[4],"ordinal":n});
    let bytes = serde_json::to_vec(&json).unwrap();
    let decoded: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, json);
    assert_eq!(decoded["ordinal"], n);
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t101, 101);
case!(t102, 102);
case!(t103, 103);
case!(t104, 104);
case!(t105, 105);
case!(t106, 106);
case!(t107, 107);
case!(t108, 108);
case!(t109, 109);
case!(t110, 110);
case!(t111, 111);
case!(t112, 112);
case!(t113, 113);
case!(t114, 114);
case!(t115, 115);
case!(t116, 116);
case!(t117, 117);
case!(t118, 118);
case!(t119, 119);
case!(t120, 120);
case!(t121, 121);
case!(t122, 122);
case!(t123, 123);
case!(t124, 124);
case!(t125, 125);
case!(t126, 126);
case!(t127, 127);
case!(t128, 128);
case!(t129, 129);
case!(t130, 130);
case!(t131, 131);
case!(t132, 132);
case!(t133, 133);
case!(t134, 134);
case!(t135, 135);
case!(t136, 136);
case!(t137, 137);
case!(t138, 138);
case!(t139, 139);
case!(t140, 140);
case!(t141, 141);
case!(t142, 142);
case!(t143, 143);
case!(t144, 144);
case!(t145, 145);
case!(t146, 146);
case!(t147, 147);
case!(t148, 148);
case!(t149, 149);
case!(t150, 150);
