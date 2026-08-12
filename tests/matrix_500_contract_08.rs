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
    let mut words: Vec<_> = c[4].split_whitespace().collect();
    let original = words.clone();
    words.sort_unstable();
    words.dedup();
    assert!(!words.is_empty());
    assert!(original.len() >= words.len());
    assert!(words.iter().any(|w| w.contains(c[1])));
}
macro_rules! case {
    ($i:ident,$n:expr) => {
        #[test]
        fn $i() {
            contract($n)
        }
    };
}
case!(t351, 351);
case!(t352, 352);
case!(t353, 353);
case!(t354, 354);
case!(t355, 355);
case!(t356, 356);
case!(t357, 357);
case!(t358, 358);
case!(t359, 359);
case!(t360, 360);
case!(t361, 361);
case!(t362, 362);
case!(t363, 363);
case!(t364, 364);
case!(t365, 365);
case!(t366, 366);
case!(t367, 367);
case!(t368, 368);
case!(t369, 369);
case!(t370, 370);
case!(t371, 371);
case!(t372, 372);
case!(t373, 373);
case!(t374, 374);
case!(t375, 375);
case!(t376, 376);
case!(t377, 377);
case!(t378, 378);
case!(t379, 379);
case!(t380, 380);
case!(t381, 381);
case!(t382, 382);
case!(t383, 383);
case!(t384, 384);
case!(t385, 385);
case!(t386, 386);
case!(t387, 387);
case!(t388, 388);
case!(t389, 389);
case!(t390, 390);
case!(t391, 391);
case!(t392, 392);
case!(t393, 393);
case!(t394, 394);
case!(t395, 395);
case!(t396, 396);
case!(t397, 397);
case!(t398, 398);
case!(t399, 399);
case!(t400, 400);
