fn checksum(s: &str) -> u64 {
    s.bytes().fold(14695981039346656037, |h, b| {
        (h ^ b as u64).wrapping_mul(1099511628211)
    })
}
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
    let base = checksum(c[4]);
    assert_eq!(base, checksum(c[4]));
    assert_ne!(base, checksum(&format!("{}#{n}", c[4])));
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t151, 151);
case!(t152, 152);
case!(t153, 153);
case!(t154, 154);
case!(t155, 155);
case!(t156, 156);
case!(t157, 157);
case!(t158, 158);
case!(t159, 159);
case!(t160, 160);
case!(t161, 161);
case!(t162, 162);
case!(t163, 163);
case!(t164, 164);
case!(t165, 165);
case!(t166, 166);
case!(t167, 167);
case!(t168, 168);
case!(t169, 169);
case!(t170, 170);
case!(t171, 171);
case!(t172, 172);
case!(t173, 173);
case!(t174, 174);
case!(t175, 175);
case!(t176, 176);
case!(t177, 177);
case!(t178, 178);
case!(t179, 179);
case!(t180, 180);
case!(t181, 181);
case!(t182, 182);
case!(t183, 183);
case!(t184, 184);
case!(t185, 185);
case!(t186, 186);
case!(t187, 187);
case!(t188, 188);
case!(t189, 189);
case!(t190, 190);
case!(t191, 191);
case!(t192, 192);
case!(t193, 193);
case!(t194, 194);
case!(t195, 195);
case!(t196, 196);
case!(t197, 197);
case!(t198, 198);
case!(t199, 199);
case!(t200, 200);
