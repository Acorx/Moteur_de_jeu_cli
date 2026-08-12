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
    let args = vec![format!("unknown-{}-{}", c[1], n)];
    let e = aetherion::Command::parse(&args).unwrap_err();
    assert!(e.message.contains("commande inconnue"));
    assert_ne!(e.exit_code, 0);
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t251, 251);
case!(t252, 252);
case!(t253, 253);
case!(t254, 254);
case!(t255, 255);
case!(t256, 256);
case!(t257, 257);
case!(t258, 258);
case!(t259, 259);
case!(t260, 260);
case!(t261, 261);
case!(t262, 262);
case!(t263, 263);
case!(t264, 264);
case!(t265, 265);
case!(t266, 266);
case!(t267, 267);
case!(t268, 268);
case!(t269, 269);
case!(t270, 270);
case!(t271, 271);
case!(t272, 272);
case!(t273, 273);
case!(t274, 274);
case!(t275, 275);
case!(t276, 276);
case!(t277, 277);
case!(t278, 278);
case!(t279, 279);
case!(t280, 280);
case!(t281, 281);
case!(t282, 282);
case!(t283, 283);
case!(t284, 284);
case!(t285, 285);
case!(t286, 286);
case!(t287, 287);
case!(t288, 288);
case!(t289, 289);
case!(t290, 290);
case!(t291, 291);
case!(t292, 292);
case!(t293, 293);
case!(t294, 294);
case!(t295, 295);
case!(t296, 296);
case!(t297, 297);
case!(t298, 298);
case!(t299, 299);
case!(t300, 300);
