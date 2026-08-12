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
    let p = std::env::temp_dir().join(format!("aetherion-matrix-{}-{}.txt", std::process::id(), n));
    std::fs::write(&p, c[4]).unwrap();
    let got = std::fs::read_to_string(&p).unwrap();
    std::fs::remove_file(&p).unwrap();
    assert_eq!(got, c[4]);
    assert!(!p.exists());
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t201, 201);
case!(t202, 202);
case!(t203, 203);
case!(t204, 204);
case!(t205, 205);
case!(t206, 206);
case!(t207, 207);
case!(t208, 208);
case!(t209, 209);
case!(t210, 210);
case!(t211, 211);
case!(t212, 212);
case!(t213, 213);
case!(t214, 214);
case!(t215, 215);
case!(t216, 216);
case!(t217, 217);
case!(t218, 218);
case!(t219, 219);
case!(t220, 220);
case!(t221, 221);
case!(t222, 222);
case!(t223, 223);
case!(t224, 224);
case!(t225, 225);
case!(t226, 226);
case!(t227, 227);
case!(t228, 228);
case!(t229, 229);
case!(t230, 230);
case!(t231, 231);
case!(t232, 232);
case!(t233, 233);
case!(t234, 234);
case!(t235, 235);
case!(t236, 236);
case!(t237, 237);
case!(t238, 238);
case!(t239, 239);
case!(t240, 240);
case!(t241, 241);
case!(t242, 242);
case!(t243, 243);
case!(t244, 244);
case!(t245, 245);
case!(t246, 246);
case!(t247, 247);
case!(t248, 248);
case!(t249, 249);
case!(t250, 250);
